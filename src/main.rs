mod api;
mod cbor;
mod cli;
mod common;
mod errors;
mod icebreakers_api;
mod middlewares;
mod node;

use api::metrics::setup_metrics_recorder;
use api::root;
use api::tx;
use axum::extract::Request;
use axum::middleware::from_fn;
use axum::routing::{get, post};
use axum::Extension;
use axum::{Router, ServiceExt};
use clap::Parser;
use cli::Args;
use cli::Config;
use errors::AppError;
use errors::BlockfrostError;
use middlewares::errors::error_middleware;
use middlewares::metrics::track_http_metrics;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::normalize_path::NormalizePathLayer;
use tracing::{error, info, warn};
use tracing_subscriber::fmt::format::Format;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let arguments = Args::parse();
    let config = Config::from_args(arguments)?;

    tracing_subscriber::fmt()
        .with_max_level(config.log_level)
        .event_format(
            Format::default()
                .with_ansi(true)
                .with_level(true)
                .with_target(false)
                .compact(),
        )
        .init();

    let max_node_connections = 8;

    let node_conn_pool = node::NodeConnPool::new(
        max_node_connections,
        &config.node_socket_path,
        config.network_magic,
    )?;

    let icebreakers_api = match config.icebreakers_config {
        Some(_) => Some(Arc::new(RwLock::new(
            icebreakers_api::IcebreakersAPI::new(&config).await?,
        ))),
        _ => {
            // echo "…" | cowsay -W 60 | sed -r 's/\\/\\\\/g ; s/^/warn!("/g ; s/$/");/g'
            warn!(" __________________________________________ ");
            warn!("/ Running in solitary mode.                \\");
            warn!("|                                          |");
            warn!("\\ You're not part of the Blockfrost fleet! /");
            warn!(" ------------------------------------------ ");
            warn!("        \\   ^__^");
            warn!("         \\  (oo)\\_______");
            warn!("            (__)\\       )\\/\\");
            warn!("                ||----w |");
            warn!("                ||     ||");
            None
        }
    };

    let prometheus_handle = Arc::new(RwLock::new(setup_metrics_recorder()));

    let app = Router::new()
        .route("/", get(root::route))
        .route("/tx/submit", post(tx::submit::route))
        .route_layer(from_fn(track_http_metrics))
        .route("/metrics", get(api::metrics::route))
        .layer(Extension(prometheus_handle))
        .layer(Extension(node_conn_pool.clone()))
        .layer(Extension(icebreakers_api))
        .layer(from_fn(error_middleware))
        .fallback(BlockfrostError::not_found());

    let app = ServiceBuilder::new()
        .layer(NormalizePathLayer::trim_trailing_slash())
        .service(app);

    let addr = format!("{}:{}", config.server_address, config.server_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("Server is listening on {}", addr);
    info!("Log level {}", config.log_level);
    info!("Mode {}", config.mode);

    tokio::spawn(node_health_check_task(node_conn_pool));

    axum::serve(listener, ServiceExt::<Request>::into_make_service(app)).await?;

    Ok(())
}

async fn node_health_check_task(node: node::NodeConnPool) {
    loop {
        // It’s enough to get a working connection from the pool, because it’s being checked then.
        let health = node.get().await.map(drop).inspect_err(|err| {
            error!(
                "Health check: cannot get a working N2C connection from the pool: {:?}",
                err
            )
        });

        let delay = tokio::time::Duration::from_secs(if health.is_ok() { 10 } else { 2 });
        tokio::time::sleep(delay).await;
    }
}

// This is a workaround for the malloc performance issues under heavy multi-threaded load for builds targetting musl, i.e. Alpine Linux
#[cfg(target_env = "musl")]
#[global_allocator]
static GLOBAL: jemalloc::Jemalloc = jemalloc::Jemalloc;
