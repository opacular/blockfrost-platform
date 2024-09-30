use crate::{common::validate_content_type, errors::BlockfrostError, node::Node};
use axum::{body::Bytes, http::HeaderMap, response::IntoResponse, Extension, Json};
use std::sync::Arc;
use tokio::sync::RwLock;

pub async fn route(
    Extension(node): Extension<Arc<RwLock<Node>>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, BlockfrostError> {
    // Allow only application/cbor content type
    validate_content_type(&headers, &["application/cbor"])?;

    // Submit transaction
    let mut node = node.write().await;
    let response = node.submit_transaction(body).await;

    Ok(Json(response))
}
