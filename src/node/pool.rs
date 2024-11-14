use crate::errors::{AppError, BlockfrostError};
use metrics::gauge;
use tracing::{error, info};

use super::connection::NodeConn;

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

pub struct NodeConnPoolManager {
    network_magic: u64,
    socket_path: String,
}

impl deadpool::managed::Manager for NodeConnPoolManager {
    type Type = NodeConn;
    type Error = AppError;

    async fn create(&self) -> Result<NodeConn, AppError> {
        // TODO: maybe use `ExponentialBackoff` from `tokio-retry`, to have at
        // least _some_ debouncing between requests, if the node is down?
        match pallas_network::facades::NodeClient::connect(&self.socket_path, self.network_magic)
            .await
        {
            Ok(conn) => {
                info!(
                    "N2C connection to node was successfully established at socket: {}",
                    self.socket_path
                );
                gauge!("cardano_node_connections").increment(1);
                Ok(NodeConn {
                    underlying: Some(conn),
                })
            }
            Err(err) => {
                error!(
                    "Failed to connect to a N2C node socket: {}: {:?}",
                    self.socket_path, err
                );
                Err(AppError::Node(err.to_string()))
            }
        }
    }

    /// Pallas decided to make the
    /// [`pallas_network::facades::NodeClient::abort`] take ownership of `self`.
    /// That’s why we need our [`NodeConn::underlying`] to be an [`Option`],
    /// because in here we only get a mutable reference. If the connection is
    /// broken, we have to call `abort`, because it joins certain multiplexer
    /// threads. Otherwise, it’s a resource leak.
    async fn recycle(
        &self,
        conn: &mut NodeConn,
        metrics: &deadpool::managed::Metrics,
    ) -> deadpool::managed::RecycleResult<AppError> {
        // Check if the connection is still viable
        match conn.ping().await {
            Ok(_) => Ok(()),
            Err(err) => {
                error!(
                    "N2C connection no longer viable: {}, {}, {:?}",
                    self.socket_path, err, metrics
                );

                gauge!("cardano_node_connections").decrement(1);

                // Take ownership of the `NodeClient` from Pallas
                // This is the only moment when `underlying` becomes `None`.
                // I should not be used again.
                let owned = conn.underlying.take().unwrap();

                // Now call `abort` to clean up their resources:
                owned.abort().await;

                // And scrap the connection from the pool:
                Err(deadpool::managed::RecycleError::Backend(AppError::Node(
                    err.to_string(),
                )))
            }
        }
    }
}
