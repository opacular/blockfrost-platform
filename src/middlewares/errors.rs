use crate::errors::BlockfrostError;
use axum::{
    body::{to_bytes, Bytes},
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use sentry::{
    protocol::{Event, Exception},
    Breadcrumb, Level,
};
use serde_json;
use std::convert::Infallible;

pub async fn error_middleware(request: Request, next: Next) -> Result<Response, Infallible> {
    let request_path = request.uri().path().to_string();
    let response = next.run(request).await;
    let status_code = response.status();

    // transform timeout to internal server error for user
    if response.status() == StatusCode::REQUEST_TIMEOUT {
        return Ok(BlockfrostError::internal_server_error_user().into_response());
    }

    // transform our custom METHOD_NOT_ALLOWED err
    if response.status() == StatusCode::METHOD_NOT_ALLOWED {
        return Ok(BlockfrostError::method_not_allowed().into_response());
    }

    // lets handle and log 5xx
    if response.status().is_server_error() {
        handle_server_error(response, &request_path, status_code).await
    } else {
        Ok(response)
    }
}

async fn handle_server_error(
    response: Response,
    request_path: &str,
    status_code: StatusCode,
) -> Result<Response, Infallible> {
    let body = response.into_body();

    match to_bytes(body, usize::MAX).await {
        Ok(bytes) => parse_and_log_error(bytes, request_path, status_code).await,
        Err(e) => {
            log_and_capture_error("Failed to read body", e, request_path, status_code);
        }
    }

    Ok(BlockfrostError::internal_server_error_user().into_response())
}

async fn parse_and_log_error(bytes: Bytes, request_path: &str, status_code: StatusCode) {
    match serde_json::from_slice::<BlockfrostError>(&bytes) {
        Ok(error_info) => {
            tracing::error!(
                "{status_code} in `{}` message: `{}`",
                request_path,
                error_info.message
            );
            log_to_sentry("|", format!("{:?}", error_info), request_path, status_code)
        }
        Err(e) => {
            println!("Failed to parse body as JSON: {:?}", e);
            log_to_sentry(
                "JSON Parse Error",
                format!("{:?}", e),
                request_path,
                status_code,
            );
            let body_str = String::from_utf8_lossy(&bytes);
            println!("Raw Body: {}", body_str);
        }
    }
}

fn log_and_capture_error(
    message: &str,
    error: impl std::fmt::Debug,
    request_path: &str,
    status_code: StatusCode,
) {
    tracing::error!(
        "{}: {:?}, Path: {}, Status: {}",
        message,
        error,
        request_path,
        status_code,
    );

    let exception = Exception {
        ty: "ServerError".to_string(),
        value: Some(format!("{:?}", error)),
        ..Default::default()
    };

    let event = Event {
        message: Some(format!(
            "{}: Path: {}, Status: {}",
            message, request_path, status_code
        )),
        level: Level::Error,
        exception: vec![exception].into(),
        ..Default::default()
    };

    sentry::capture_event(event);
}

fn log_to_sentry(context: &str, detail: String, request_path: &str, status_code: StatusCode) {
    let breadcrumb = Breadcrumb {
        message: Some(format!("Request at {}", request_path)),
        category: Some("request".into()),
        level: Level::Info,
        ..Default::default()
    };

    sentry::add_breadcrumb(breadcrumb);

    let event = Event {
        message: Some(format!("{} - {}: {}", status_code, context, detail)),
        level: Level::Error,
        ..Default::default()
    };

    sentry::capture_event(event);
}
