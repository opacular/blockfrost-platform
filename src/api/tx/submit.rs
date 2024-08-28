use crate::{common::validate_content_type, errors::BlockfrostError, node::Node};
use axum::{body::Bytes, http::HeaderMap, response::IntoResponse, Extension};
use std::sync::Arc;

pub async fn route(
    Extension(node): Extension<Arc<Node>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, BlockfrostError> {
    validate_content_type(&headers, &["application/cbor"])?;

    let response = node.submit_transaction(body.to_vec()).await?;

    Ok(response)
}
