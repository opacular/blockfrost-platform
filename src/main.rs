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
use config::Args;
use errors::AppError;
use errors::BlockfrostError;
use middlewares::errors::error_middleware;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::normalize_path::NormalizePathLayer;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let arguments = Args::parse();
    let config = config::load_config(arguments.config)?;

    tracing_subscriber::fmt()
        .with_max_level(config.server.log_level)
        .init();

    let node_instance = node::Node::new(&config.server.relay, config.server.network_magic).await?;
    let node = Arc::new(RwLock::new(node_instance));

    let app = Router::new()
        .route("/", get(root::route))
        .route("/tx/submit", post(tx::submit::route))
        .layer(Extension(node))
        .layer(from_fn(error_middleware))
        .fallback(BlockfrostError::not_found());

    let app = ServiceBuilder::new()
        .layer(NormalizePathLayer::trim_trailing_slash())
        .service(app);

    let listener = tokio::net::TcpListener::bind(&config.server.address).await?;

    info!("Server is listening on {}", config.server.address);
    info!("Log level {}", config.server.log_level);

    axum::serve(listener, ServiceExt::<Request>::into_make_service(app)).await?;

    Ok(())
}
