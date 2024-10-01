use crate::errors::{AppError, BlockfrostError};
use axum::body::Bytes;
use pallas_crypto::hash::Hasher;
use pallas_network::{
    facades::NodeClient,
    miniprotocols::localtxsubmission::{EraTx, Response},
};
use tracing::{info, warn};

pub struct Node {
    client: NodeClient,
}

impl Node {
    /// Creates a new `Node` instance
    pub async fn new(socket: &str, network_magic: u64) -> Result<Node, AppError> {
        info!("Connecting to node socket {} ...", socket);

        let client = NodeClient::connect(socket, network_magic).await?;

        info!("Connection to node was successfully established.");

        Ok(Node { client })
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
}
