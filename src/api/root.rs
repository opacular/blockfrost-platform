use axum::Extension;
use axum::Json;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::node::Node;

#[derive(Serialize)]
pub struct Response {
    pub name: String,
    pub version: String,
    pub node_version: String,
    pub errors: Option<Vec<String>>,
    pub healthy: bool,
}

pub async fn route(Extension(node): Extension<Arc<RwLock<Node>>>) -> Json<Response> {
    let mut node = node.write().await;
    let version = node.version().await;

    let mut found_errors = Vec::new();
    let version = version.unwrap_or_else(|_| {
        found_errors.push("Failed to determine node version".to_string());
        "unknown".to_string()
    });

    let response = Response {
        name: "blockfrost-platform".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        node_version: version,
        healthy: found_errors.is_empty(),
        errors: if found_errors.is_empty() {
            None
        } else {
            Some(found_errors)
        },
    };

    Json(response)
}
