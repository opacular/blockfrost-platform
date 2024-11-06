use crate::errors::{AppError, BlockfrostError};
use chrono::{Duration, TimeZone, Utc};
use metrics::gauge;
use pallas_crypto::hash::Hasher;
use pallas_network::{
    miniprotocols,
    miniprotocols::{
        localstate,
        localtxsubmission::{EraTx, Response},
    },
};
use pallas_traverse::wellknown;
use std::boxed::Box;
use std::pin::Pin;
use tracing::{error, info, warn};

/// This represents a pool of Node2Client connections to a single `cardano-node`.
///
/// It can be safely cloned to multiple threads, while still sharing the same
/// set of underlying connections to the node.
#[derive(Clone)]
pub struct NodeConnPool {
    underlying: deadpool::managed::Pool<NodeConnPoolManager>,
}

/// Our wrapper around [`pallas_network::facades::NodeClient`]. If you only use
/// this, you won’t get any deadlocks, inconsistencies, etc.
pub struct NodeConn {
    /// Note: this is an [`Option`] *only* to satisfy the borrow checker. It’s
    /// *always* [`Some`]. See [`NodeConnPoolManager::recycle`] for an
    /// explanation.
    underlying: Option<pallas_network::facades::NodeClient>,
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

impl NodeConn {
    /// Submits a transaction to the connected Cardano node.
    pub async fn submit_transaction(&mut self, tx: String) -> Result<String, BlockfrostError> {
        let tx = hex::decode(tx).map_err(|e| BlockfrostError::custom_400(e.to_string()))?;
        let txid = hex::encode(Hasher::<256>::hash_cbor(&tx));

        let era_tx = EraTx(6, tx);

        // Connect to the node
        let submission_client = self.underlying.as_mut().unwrap().submission();

        // Submit the transaction
        match submission_client.submit_tx(era_tx).await {
            Ok(Response::Accepted) => {
                info!("Transaction accepted by the node {}", txid);
                Ok(txid)
            }
            Ok(Response::Rejected(reason)) => {
                let reason_hex = hex::encode(&reason.0);
                warn!("Transaction was rejected: {}", reason_hex);
                Err(BlockfrostError::custom_400(reason_hex))
            }
            Err(e) => {
                warn!("Error during transaction submission: {:?}", e);
                Err(BlockfrostError::custom_400(e.to_string()))
            }
        }
    }

    /// We always have to release the [`localstate::GenericClient`], even on errors,
    /// otherwise `cardano-node` stalls. If you use this function, it’s handled for you.
    async fn with_statequery<A, F>(&mut self, action: F) -> Result<A, BlockfrostError>
    where
        F: for<'a> FnOnce(
            &'a mut localstate::GenericClient,
        ) -> Pin<
            Box<dyn std::future::Future<Output = Result<A, BlockfrostError>> + 'a + Sync + Send>,
        >,
    {
        // Acquire the client
        let client = self.underlying.as_mut().unwrap().statequery();
        client.acquire(None).await?;

        // Run the action and ensure the client is released afterwards
        let result = action(client).await;

        // Always release the client, even if action fails
        if let Err(e) = client.send_release().await {
            warn!("Failed to release client: {:?}", e);
        }

        result
    }

    /// Pings the node, e.g. to see if the connection is still alive.
    pub async fn ping(&mut self) -> Result<(), BlockfrostError> {
        // FIXME: we should be able to use `miniprotocols::keepalive`
        // (cardano-cli does), but for some reason it’s not added to
        // `NodeClient`? Let’s try to acquire a local state client instead:

        self.with_statequery(|_| Box::pin(async { Ok(()) })).await
    }

    /// Reports the sync progress of the node.
    pub async fn sync_progress(&mut self) -> Result<SyncProgress, BlockfrostError> {
        async fn action(
            generic_client: &mut localstate::GenericClient,
        ) -> Result<SyncProgress, BlockfrostError> {
            let current_era = localstate::queries_v16::get_current_era(generic_client).await?;

            let epoch =
                localstate::queries_v16::get_block_epoch_number(generic_client, current_era)
                    .await?;

            let geneses =
                localstate::queries_v16::get_genesis_config(generic_client, current_era).await?;
            let genesis = geneses.first().ok_or_else(|| {
                BlockfrostError::internal_server_error("Expected at least one genesis".to_string())
            })?;

            let system_start = localstate::queries_v16::get_system_start(generic_client).await?;
            let chain_point = localstate::queries_v16::get_chain_point(generic_client).await?;
            let slot = chain_point.slot_or_default();

            // FIXME: this is debatable, because it won’t work for custom networks; we should rather
            // get this information by calling `Ouroboros.Consensus.HardFork.History.Qry.slotToWallclock`
            // like both cardano-cli (through cardano-api) and Ogmios do, but it’s not implemented
            // in pallas_network yet.
            let wellknown_genesis = wellknown::GenesisValues::from_magic(
                genesis.network_magic.into(),
            )
            .ok_or_else(|| {
                BlockfrostError::internal_server_error(format!(
                    "Only well-known networks are supported (unsupported network magic: {})",
                    genesis.network_magic
                ))
            })?;

            let year: i32 = system_start.year.try_into().map_err(|e| {
                BlockfrostError::internal_server_error(format!("Failed to convert year: {}", e))
            })?;

            let base_date = Utc
                .with_ymd_and_hms(year, 1, 1, 0, 0, 0)
                .single()
                .ok_or_else(|| {
                    BlockfrostError::internal_server_error("Invalid base date".to_string())
                })?;

            let days = Duration::days((system_start.day_of_year - 1).into());

            let nanoseconds: i64 = (system_start.picoseconds_of_day / 1_000)
                .try_into()
                .map_err(|e| {
                    BlockfrostError::internal_server_error(format!(
                        "Failed to convert picoseconds: {}",
                        e
                    ))
                })?;

            let duration_ns = Duration::nanoseconds(nanoseconds);

            let utc_start = base_date + days + duration_ns;

            let slot_time_secs: i64 = wellknown_genesis
                .slot_to_wallclock(slot)
                .try_into()
                .map_err(|e| {
                    BlockfrostError::internal_server_error(format!(
                        "Failed to convert slot time: {}",
                        e
                    ))
                })?;

            let utc_slot = Utc
                .timestamp_opt(slot_time_secs, 0)
                .single()
                .ok_or_else(|| {
                    BlockfrostError::internal_server_error("Invalid slot timestamp".to_string())
                })?;

            let utc_now = Utc::now();

            let utc_slot_capped = std::cmp::min(utc_now, utc_slot);

            let tolerance = 60; // [s]
            let percentage = if (utc_now - utc_slot_capped).num_seconds() < tolerance {
                1.0
            } else {
                let network_duration = (utc_now - utc_start).num_seconds() as f64;
                let duration_up_to_slot = (utc_slot_capped - utc_start).num_seconds() as f64;
                duration_up_to_slot / network_duration
            };

            let block = match chain_point {
                miniprotocols::Point::Origin => String::new(),
                miniprotocols::Point::Specific(_, block) => hex::encode(&block),
            };

            Ok(SyncProgress {
                percentage,
                era: current_era,
                epoch,
                slot,
                block,
            })
        }

        self.with_statequery(|generic_client: &mut localstate::GenericClient| {
            Box::pin(action(generic_client))
        })
        .await
    }
}

#[derive(serde::Serialize)]
pub struct SyncProgress {
    percentage: f64,
    era: u16,
    epoch: u32,
    slot: u64,
    block: String,
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
                // Take ownership of the `NodeClient` from Pallas
                let owned = conn.underlying.take().unwrap();
                // This is the only moment when `underlying` becomes `None`. But
                // it will never be used again.
                gauge!("cardano_node_connections").decrement(1);
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
