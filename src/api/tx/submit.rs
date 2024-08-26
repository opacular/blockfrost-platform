use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct Response {
    pub name: String,
    pub version: String,
    pub healthy: bool,
}

pub async fn route() -> Json<Response> {
    let response = Response {
        name: "blockfrost-instance".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        healthy: true,
    };

    Json(response)
}
