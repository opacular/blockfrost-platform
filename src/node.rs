use pallas_network::facades::PeerClient;
// use pallas_network::miniprotocols::txsubmission::{EraTxBody, TxIdAndSize};

use crate::errors::BlockfrostError;

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
}
