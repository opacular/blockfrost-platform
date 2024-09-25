use crate::{cli::Config, errors::AppError};
use reqwest::Client;
use serde_json::json;
use tracing::{error, info};

pub struct IcebreakersAPI {
    client: Client,
    url: String,
    secret: String,
    reward_address: String,
}

const API_URL: &str = "https://icebreakers-api.blockfrost.io";

impl IcebreakersAPI {
    /// Creates a new `Icebreakers` instance
    pub async fn new(config: &Config) -> Result<IcebreakersAPI, AppError> {
        info!("Connecting to icebreakers API...");

        let client = Client::new();
        let url = format!("{}/health", API_URL);
        let mut icebreakers = IcebreakersAPI {
            client,
            url,
            secret: config.secret.clone(),
            reward_address: config.reward_address.clone(),
        };

        // Register with icebreakers
        match icebreakers.register().await {
            Ok(info) => info!(
                "Connection to icebreakers API was successfully established. {}",
                info
            ),
            Err(e) => {
                error!("Failed to register with icebreakers: {}", e);
                panic!()
            }
        }

        Ok(icebreakers)
    }

    /// Submits a transaction to the connected Cardano node.
    pub async fn register(&mut self) -> Result<String, AppError> {
        info!("Registering with icebreakers...");

        let json_body = json!({
            "secret": self.secret,
            "reward_address": self.reward_address,
        });

        let _ = self
            .client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .json(&json_body)
            .send()
            .await;

        // handle error here

        Ok("Registered with icebreakers".to_string())
    }
}
