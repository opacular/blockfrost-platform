use super::pool_manager::NodeConnPoolManager;
use crate::{
    cli::Config,
    errors::{AppError, BlockfrostError},
};

/// This represents a pool of Node2Client connections to a single `cardano-node`.
///
/// It can be safely cloned to multiple threads, while still sharing the same
/// set of underlying connections to the node.
#[derive(Clone)]
pub struct NodeConnPool {
    underlying: deadpool::managed::Pool<NodeConnPoolManager>,
}

impl NodeConnPool {
    /// Creates a new pool of [`NodeConn`] connections.
    pub fn new(config: &Config) -> Result<Self, AppError> {
        let manager = NodeConnPoolManager {
            network_magic: config.network_magic,
            socket_path: config.node_socket_path.to_string(),
        };
        let underlying = deadpool::managed::Pool::builder(manager)
            .max_size(config.max_pool_connections)
            .build()
            .map_err(|err| AppError::Node(err.to_string()))?;
        Ok(Self { underlying })
    }

    /// Borrows a single [`NodeConn`] connection from the pool.
    ///
    /// TODO: it should probably return an [`AppError`], but with
    /// [`BlockfrostError`] it’s much easier to integrate in request handlers.
    /// We don’t convert them automatically.
    pub async fn get(
        &self,
    ) -> Result<deadpool::managed::Object<NodeConnPoolManager>, BlockfrostError> {
        self.underlying
            .get()
            .await
            .map_err(|err| BlockfrostError::internal_server_error(format!("NodeConnPool: {}", err)))
    }
}
