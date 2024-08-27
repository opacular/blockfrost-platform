mod api;
mod config;
mod errors;
mod middlewares;
mod node;

use std::sync::Arc;

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
use errors::BlockfrostError;
use middlewares::errors::error_middleware;
use tower::ServiceBuilder;
use tower_http::normalize_path::NormalizePathLayer;

#[tokio::main]
async fn main() {
    let arguments = Args::parse();
    let config = match config::load_config(arguments.config) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("\nError loading configuration: {}\n", e);
            panic!();
        }
    };

    tracing_subscriber::fmt()
        .with_max_level(config.server.log_level)
        .init();

    let node = node::Node::new(&config.node.endpoint, config.server.network_magic)
        .await
        .expect("Failed to connect to the node");

    let shared_node = Arc::new(node);

    let app = Router::new()
        .route("/", get(root::route))
        .route("/tx/submit", post(tx::submit::route))
        .layer(Extension(shared_node))
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
        .expect("Failed to start the server");
}
