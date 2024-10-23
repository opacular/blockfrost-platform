use crate::{common::validate_content_type, errors::BlockfrostError, node::Node};
use axum::{http::HeaderMap, response::IntoResponse, Extension, Json};
use std::sync::Arc;
use tokio::sync::RwLock;

pub async fn route(
    Extension(node): Extension<Arc<RwLock<Node>>>,
    headers: HeaderMap,
    body: String,
) -> Result<impl IntoResponse, BlockfrostError> {
    // Allow only application/cbor content type
    validate_content_type(&headers, &["application/cbor"])?;

    // Submit transaction
    let node = node.write().await;
    let response = node.submit_transaction(body).await?;

    Ok(Json(response))
}
