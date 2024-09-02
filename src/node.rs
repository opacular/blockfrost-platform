use crate::errors::BlockfrostError;
use pallas_network::{
    facades::PeerClient,
    miniprotocols::txsubmission::{EraTxBody, EraTxId, TxIdAndSize},
};

pub struct Node {
    client: PeerClient,
}

impl Node {
    /// Creates a new `Node` instance and connects to the specified Cardano network.
    pub async fn new(url: &str, network_magic: u64) -> Result<Node, BlockfrostError> {
        let client = PeerClient::connect(url, network_magic).await?;

        Ok(Self { client })
    }

    /// Submits a transaction to the connected Cardano node.
    pub async fn submit_transaction(
        &mut self,
        tx_bytes: Vec<u8>,
    ) -> Result<String, BlockfrostError> {
        let tx_hash: [u8; 32] = tx_bytes[..].try_into()?;
        let tx_size = tx_bytes.len() as u32;

        let ids_and_size = vec![TxIdAndSize(EraTxId(4, tx_hash.to_vec()), tx_size)];
        let tx_body = vec![EraTxBody(4, tx_bytes.clone())];

        let client_txsub = self.client.txsubmission();

        client_txsub.send_init().await?;
        client_txsub.reply_tx_ids(ids_and_size).await?;
        client_txsub.reply_txs(tx_body).await?;

        match client_txsub.next_request().await? {
            pallas_network::miniprotocols::txsubmission::Request::TxIds(ack, _) => {
                client_txsub.send_done().await?;
                Ok(ack.to_string())
            }
            _ => Err(BlockfrostError::internal_server_error(
                "Unexpected response from node".to_string(),
            )),
        }
    }
}
