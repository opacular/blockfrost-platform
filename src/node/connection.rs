use crate::cbor::haskell_types::TxValidationError;
use crate::errors::BlockfrostError;
use crate::node::SyncProgress;
use chrono::{Duration, TimeZone, Utc};
use pallas_codec::minicbor::{display, Decoder};
use pallas_crypto::hash::Hasher;
use pallas_network::miniprotocols::localtxsubmission::{EraTx, Response};
use pallas_network::multiplexer::Error;
use pallas_network::{miniprotocols, miniprotocols::localstate};
use pallas_traverse::wellknown;
use std::boxed::Box;
use std::pin::Pin;
use tracing::{info, warn};

/// Our wrapper around [`pallas_network::facades::NodeClient`]. If you only use
/// this, you won’t get any deadlocks, inconsistencies, etc.
pub struct NodeConn {
    /// Note: this is an [`Option`] *only* to satisfy the borrow checker. It’s
    /// *always* [`Some`]. See [`NodeConnPoolManager::recycle`] for an
    /// explanation.
    underlying: Option<pallas_network::facades::NodeClient>,
}

impl NodeConn {
    pub(in crate::node) fn new(underlying: pallas_network::facades::NodeClient) -> Self {
        Self {
            underlying: Some(underlying),
        }
    }

    pub async fn abort(&mut self) {
        // Take ownership of the `NodeClient` from Pallas
        // This is the only moment when `underlying` becomes `None`.
        // I should not be used again.
        let owned = self.underlying.take().unwrap();

        // Now call `abort` to clean up their resources:
        owned.abort().await;
    }

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
                let reason = reason.0;

                let msg_res = Self::try_decode_error(&reason);

                let error_message = format!("Transaction rejected with reason: {:?}", msg_res);

                warn!(error_message);

                Err(BlockfrostError::custom_400(error_message))
            }
            Err(e) => {
                let error_message = format!("Error during transaction submission: {:?}", e);

                Err(BlockfrostError::custom_400(error_message))
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

    fn try_decode_error(buffer: &[u8]) -> Result<Option<TxValidationError>, Error> {
        let maybe_error = Decoder::new(buffer).decode();

        match maybe_error {
            Ok(error) => Ok(Some(error)),
            Err(err) => {
                let buffer_display = display(buffer);
                warn!(
                    "Failed to decode error: {:?}, buffer: {}",
                    err, buffer_display
                );

                // Decoding failures are not errors, but some missing implementation or mis-implementations on our side.
                // A decoding failure is a bug in our code, not a bug in the node.
                // It should not effect the program flow, but should be logged and reported.
                Err(Error::Decoding(err.to_string()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_decode_error() {
        let buffer = [
            130, 2, 129, 130, 6, 130, 130, 1, 130, 0, 131, 6, 27, 0, 0, 0, 2, 54, 42, 119, 48, 27,
            0, 0, 0, 2, 83, 185, 193, 29, 130, 1, 130, 0, 131, 5, 26, 0, 2, 139, 253, 24, 173,
        ];
        let error = NodeConn::try_decode_error(&buffer).unwrap();

        assert!(error.is_some());
    }
}
