use crate::errors::{AppError, BlockfrostError};
use pallas_network::miniprotocols::txsubmission::Request::TxIds;
use pallas_network::{
    facades::PeerClient,
    miniprotocols::txsubmission::{EraTxBody, EraTxId, TxIdAndSize},
};
use tracing::info;

pub struct Node {
    client: PeerClient,
}

impl Node {
    /// Creates a new `Node` instance
    pub async fn new(url: &str, network_magic: u64) -> Result<Node, AppError> {
        info!("Connecting to node {}", url);

        let client = PeerClient::connect(url, network_magic)
            .await
            .map_err(|e| AppError::NodeError(format!("Failed to connect to node: {}", e)))?;

        info!("Connection to node was successfully established");

        Ok(Node { client })
    }

    /// Submits a transaction to the connected Cardano node.
    pub async fn submit_transaction(
        &mut self,
        tx_bytes: Vec<u8>,
    ) -> Result<String, BlockfrostError> {
        info!("Submitting transaction to node");

        let tx_size = tx_bytes.len() as u32;
        let ids_and_size = vec![TxIdAndSize(EraTxId(4, tx_bytes.clone()), tx_size)];
        let tx_body = vec![EraTxBody(4, tx_bytes)];

        let client_txsub = self.client.txsubmission();

        client_txsub.send_init().await?;
        client_txsub.reply_tx_ids(ids_and_size).await?;
        client_txsub.reply_txs(tx_body).await?;

        match client_txsub.next_request().await {
            // successfully received ack
            Ok(TxIds(ack, _)) => {
                client_txsub.send_done().await?;
                Ok(ack.to_string())
            }
            Ok(_) => {
                // unexpected response, handle error
                client_txsub.send_done().await?;
                Err(BlockfrostError::internal_server_error(
                    "Unexpected response from node".to_string(),
                ))
            }
            Err(e) => {
                client_txsub.send_done().await?;
                Err(BlockfrostError::from(e))
            }
        }
    }
}
