use std::str::Bytes;

use crate::errors::BlockfrostError;
use pallas_crypto::hash::Hash;
use pallas_network::{
    facades::PeerClient,
    miniprotocols::txsubmission::{EraTxBody, EraTxId, TxIdAndSize},
};

pub struct Node {
    network_magic: u64,
    client: PeerClient,
}

impl Node {
    /// Creates a new `Node` instance and connects to the specified Cardano network.
    pub async fn new(url: &str, network_magic: u64) -> Result<Self, BlockfrostError> {
        let client = PeerClient::connect(url, network_magic)
            .await
            .map_err(|err| BlockfrostError::internal_server_error(err.to_string()))?;

        Ok(Self {
            client,
            network_magic,
        })
    }

    /// Submits a transaction to the connected Cardano node.
    pub async fn submit_transaction(
        &mut self,
        tx_bytes: Vec<u8>,
    ) -> Result<String, BlockfrostError> {
        // let tx_hash: Bytes = Bytes::from(Hash::hash_bytes(&tx_bytes).as_ref());
        let tx_size = tx_bytes.len() as u32;

        // let ids_and_size = vec![TxIdAndSize(EraTxId(4, tx_hash.to_vec()), tx_size)];
        let tx_body = vec![EraTxBody(4, tx_bytes.clone())];

        let client_txsub = self.client.txsubmission();

        client_txsub.send_init().await.unwrap();
        // client_txsub.reply_tx_ids(ids_and_size).await.unwrap();
        client_txsub.reply_txs(tx_body).await.unwrap();

        match client_txsub.next_request().await.unwrap() {
            pallas_network::miniprotocols::txsubmission::Request::TxIds(ack, _) => {
                client_txsub.send_done().await.unwrap();
                Ok(ack.to_string())
            }
            _ => Err(BlockfrostError::internal_server_error(
                "Unexpected response from node".to_string(),
            )),
        }
    }
}
