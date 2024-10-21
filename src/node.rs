use crate::errors::BlockfrostError;
use axum::body::Bytes;
use pallas_crypto::hash::Hasher;
use pallas_network::{
    facades::NodeClient,
    miniprotocols::localtxsubmission::{EraTx, Response},
};
use tracing::{info, warn};

pub struct Node {
    network_magic: u64,
    socket: String,
}

impl Node {
    /// Creates a new `Node` instance
    pub fn new(socket: &str, network_magic: u64) -> Self {
        Self {
            socket: socket.to_string(),
            network_magic,
        }
    }

    /// Establishes a new NodeClient connection.
    async fn connect(&self) -> Result<NodeClient, BlockfrostError> {
        info!("Connecting to node socket {} ...", self.socket);

        match NodeClient::connect(&self.socket, self.network_magic).await {
            Ok(client) => {
                info!("Connection to node was successfully established.");
                Ok(client)
            }
            Err(e) => {
                warn!("Failed to connect to node: {:?}", e);
                Err(BlockfrostError::custom_400(e.to_string()))
            }
        }
    }

    /// Submits a transaction to the connected Cardano node.
    pub async fn submit_transaction(&self, tx: Bytes) -> Result<String, BlockfrostError> {
        let tx_vec = tx.to_vec();
        let txid = hex::encode(Hasher::<256>::hash_cbor(&tx_vec));

        let era_tx = EraTx(6, tx_vec);

        // Connect to the node
        let mut client = self.connect().await?;
        let submission_client = client.submission();

        // Submit the transaction
        match submission_client.submit_tx(era_tx).await {
            Ok(Response::Accepted) => {
                info!("Transaction accepted by the node.");
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
}
