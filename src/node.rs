use crate::errors::{AppError, BlockfrostError};
use pallas::network::multiplexer::Plexer;
use tokio::net::UnixStream;

use pallas_network::{
    facades::{NodeClient, PeerClient},
    miniprotocols::{
        localtxsubmission::{Client, EraTx, Response},
        txsubmission::{EraTxId, TxIdAndSize},
    },
    multiplexer::Bearer,
};
use tokio::net::TcpStream;
use tracing::info;

pub struct Node {
    client: Client,
}

impl Node {
    /// Creates a new `Node` instance
    pub async fn new(socket: &str, network_magic: u64) -> Result<Node, AppError> {
        info!("Connecting to node {} ...", socket);

        // Connect to the local node via Unix domain socket
        let socket = UnixStream::connect(socket).await?;
        // info!("Connected to node at {}", node_socket);

        // Create a bearer from the Unix socket
        let bearer = Bearer::Unix(socket);

        // Initialize the Plexer with the bearer
        let mut plexer = Plexer::new(bearer);
        info!("Plexer set up successfully.");

        // Use the LocalTxSubmission protocol channel (protocol ID 7)
        let channel = plexer.subscribe_client(7);
        info!("Channel for LocalTxSubmission protocol created.");

        // Spawn the plexer tasks
        let plexer_tasks = plexer.spawn();
        info!("Plexer tasks spawned.");

        // Create a TxSubmissionClient instance using the channel
        let mut client = Client::new(channel);
        info!("TxSubmissionClient initialized.");

        info!("Connection to node was successfully established.");

        Ok(Node { client })
    }

    /// Submits a transaction to the connected Cardano node.
    pub async fn submit_transaction(
        &mut self,
        tx_bytes: Vec<u8>,
    ) -> Result<String, BlockfrostError> {
        info!("Submitting transaction to node.");

        let tx = EraTx(5, tx_bytes.clone());

        let response = self.client.submit_tx(tx).await.unwrap();

        match response {
            Response::Accepted => println!("Transaction accepted by the node."),
            Response::Rejected(reason) => println!("Transaction rejected: {:?}", reason),
        }

        Ok("Transaction submitted.".to_string())
    }
}
