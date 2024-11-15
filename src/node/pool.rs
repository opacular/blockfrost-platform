use super::pool_manager::NodeConnPoolManager;
use crate::errors::AppError;

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
    pub fn new(
        pool_max_size: usize,
        socket_path: &str,
        network_magic: u64,
    ) -> Result<Self, AppError> {
        let manager = NodeConnPoolManager {
            network_magic,
            socket_path: socket_path.to_string(),
        };
        let underlying = deadpool::managed::Pool::builder(manager)
            .max_size(pool_max_size)
            .build()
            .map_err(|err| AppError::Node(err.to_string()))?;
        Ok(Self { underlying })
    }

    /// Borrows a single [`NodeConn`] connection from the pool.
    pub async fn get(&self) -> Result<deadpool::managed::Object<NodeConnPoolManager>, AppError> {
        self.underlying
            .get()
            .await
            .map_err(|err| AppError::Node(format!("NodeConnPool: {}", err)))
    }
}
