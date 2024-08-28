use crate::errors::BlockfrostError;
use pallas_network::{
    facades::PeerClient,
    // miniprotocols::txsubmission::{EraTxBody, TxIdAndSize},
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
    pub async fn submit_transaction(&self, tx_bytes: Vec<u8>) -> Result<String, BlockfrostError> {
        println!("submit_transaction  {:?}", tx_bytes);

        // let tx_hash = pallas_crypto::hash::Hash::from(tx_bytes.clone());
        // let mempool = vec![(tx_hash, tx_bytes)];
        // let client_txsub = self.client.txsubmission();

        // // Initialize the transaction submission protocol
        // client_txsub.send_init().await.unwrap();

        // // Send the transaction ID and size to the node
        // let ids_and_size: Vec<TxIdAndSize> = mempool
        //     .iter()
        //     .map(|(h, b)| {
        //         TxIdAndSize(
        //             pallas_network::miniprotocols::txsubmission::EraTxId(4, h.clone()),
        //             b.len() as u32,
        //         )
        //     })
        //     .collect();

        // client_txsub.reply_tx_ids(ids_and_size).await.unwrap();

        // // Send the actual transaction to the node
        // let txs_to_send: Vec<EraTxBody> =
        //     mempool.into_iter().map(|(_, b)| EraTxBody(4, b)).collect();
        // client_txsub.reply_txs(txs_to_send).await.unwrap();

        // // Wait for acknowledgment from the node
        // match client_txsub.next_request().await.unwrap() {
        //     pallas_network::miniprotocols::txsubmission::Request::TxIds(ack, _) => Ok(ack),
        //     _ => Err("Unexpected response from node".into()),
        // }
        //
        Ok("aaaa".to_string())
    }
}
