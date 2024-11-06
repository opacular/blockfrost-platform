use crate::node::{NodeConnPool, SyncProgress};
use axum::{Extension, Json};
use serde::Serialize;
use tracing::error;

#[derive(Serialize)]
pub struct Response {
    pub name: String,
    pub version: String,
    pub sync_progress: Option<SyncProgress>,
    pub healthy: bool,
    pub errors: Vec<String>,
}

pub async fn route(Extension(node): Extension<NodeConnPool>) -> Json<Response> {
    let mut errors = vec![];

    let sync_progress = match node.get().await {
        Ok(mut node) => node.sync_progress().await,
        Err(err) => Err(err),
    }
    .inspect_err(|err| {
        error!("{:?}", err);
        errors.push("Failed to determine sync_percentage".to_string());
    })
    .ok();

    let response = Response {
        name: "blockfrost-platform".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        sync_progress,
        healthy: errors.is_empty(),
        errors,
    };

    Json(response)
}
