mod api;
mod common;
mod config;
mod errors;
mod middlewares;
mod node;

use api::root;
use api::tx;
use axum::extract::Request;
use axum::middleware::from_fn;
use axum::routing::{get, post};
use axum::Extension;
use axum::{Router, ServiceExt};
use clap::Parser;
use colored::Colorize;
use config::Args;
use errors::AppError;
use errors::BlockfrostError;
use middlewares::errors::error_middleware;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::normalize_path::NormalizePathLayer;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let arguments = Args::parse();
    let config =
        config::load_config(arguments.config).map_err(|e| AppError::ConfigError(e.to_string()))?;

    tracing_subscriber::fmt()
        .with_max_level(config.server.log_level)
        .init();

    let node = node::Node::new(&config.node.endpoint, config.server.network_magic)
        .await
        .map_err(|e| AppError::NodeError(e.to_string()))?;

    let app = Router::new()
        .route("/", get(root::route))
        .route("/tx/submit", post(tx::submit::route))
        .layer(Extension(Arc::new(node)))
        .layer(from_fn(error_middleware))
        .fallback(BlockfrostError::not_found());

    let app = ServiceBuilder::new()
        .layer(NormalizePathLayer::trim_trailing_slash())
        .service(app);

    let listener = tokio::net::TcpListener::bind(&config.server.address)
        .await
        .expect("Failed to bind to address");

    println!(
        "{}",
        format!(
            "\nAddress: 🌍 http://{}\n\
             Log Level: 📘 {}\n",
            config.server.address, config.server.log_level,
        )
        .white()
        .bold()
    );

    axum::serve(listener, ServiceExt::<Request>::into_make_service(app))
        .await
        .map_err(|e| AppError::ServerError(e.to_string()))?;

    Ok(())
}
