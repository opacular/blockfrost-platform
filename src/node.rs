use crate::errors::{AppError, BlockfrostError};
use axum::body::Bytes;
use pallas_crypto::hash::Hasher;
use pallas_network::{
    facades::NodeClient,
    miniprotocols::{
        handshake::{self, Confirmation},
        localtxsubmission::{EraTx, Response},
    },
};
use tracing::{info, warn};

pub struct Node {
    client: NodeClient,
    network_magic: u64,
}

impl Node {
    /// Creates a new `Node` instance
    pub async fn new(socket: &str, network_magic: u64) -> Result<Node, AppError> {
        info!("Connecting to node socket {} ...", socket);

        let client = NodeClient::connect(socket, network_magic).await?;

        info!("Connection to node was successfully established.");
        Ok(Node {
            client,
            network_magic,
        })
    }

    /// Submits a transaction to the connected Cardano node.
    pub async fn submit_transaction(&mut self, tx: Bytes) -> Result<String, BlockfrostError> {
        info!("Submitting transaction to node.");

        let tx_vec = tx.to_vec();
        let txid = hex::encode(Hasher::<256>::hash_cbor(&tx_vec));
        let era_tx = EraTx(6, tx_vec);

        match self.client.submission().submit_tx(era_tx).await? {
            Response::Accepted => Ok(txid),
            Response::Rejected(reason) => {
                warn!("Transaction was rejected: {}", hex::encode(&reason.0));

                Err(BlockfrostError::custom_400(format!(
                    "Transaction was rejected: {}",
                    hex::encode(&reason.0)
                )))
            }
        }
    }

    // Gets the node version from the connected Cardano node.
    pub async fn version(&mut self) -> Result<String, BlockfrostError> {
        info!("Getting version of the node.");

        let versions = handshake::n2c::VersionTable::v10_and_above(self.network_magic);

        // Perform the handshake and retrieve the node version in one step
        let confirmation = self.client.handshake().handshake(versions).await?;

        // Extract the node version after successful handshake
        match confirmation {
            Confirmation::Accepted(handshake_version, _version_data) => {
                info!("Node version: {:?}", handshake_version);
                Ok(format!("{:?}", handshake_version)) // Convert version to string format
            }
            Confirmation::Rejected(reason) => Err(BlockfrostError::internal_server_error(format!(
                "Failed to get the version: {:?}",
                &reason
            ))),
            Confirmation::QueryReply(_) => Err(BlockfrostError::internal_server_error(
                "Failed to get the version".to_string(),
            )),
        }
    }
}
