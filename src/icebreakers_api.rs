use crate::{cli::Config, errors::AppError};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use tracing::info;

pub struct IcebreakersAPI {
    client: Client,
    base_url: String,
    secret: String,
    mode: String,
    port: u16,
    reward_address: String,
}

#[derive(Deserialize)]
struct ErrorResponse {
    reason: String,
    details: String,
}

const API_DEV_URL: &str = "https://api-dev.icebreakers.blockfrost.io";
// const API_URL: &str = "https://icebreakers-api.blockfrost.io";

impl IcebreakersAPI {
    /// Creates a new `IcebreakersAPI` instance
    pub async fn new(config: &Config) -> Result<Self, AppError> {
        info!("Connecting to Icebreakers API...");

        let client = Client::new();
        let base_url = API_DEV_URL.to_string();

        let icebreakers_api = IcebreakersAPI {
            client,
            base_url,
            secret: config.secret.clone(),
            mode: config.mode.to_string(),
            port: config.server_port,
            reward_address: config.reward_address.clone(),
        };

        if let Err(e) = icebreakers_api.register().await {
            return Err(e);
        } else {
            info!("Successfully registered with Icebreakers API.");
        }

        Ok(icebreakers_api)
    }

    /// Registers with the Icebreakers API
    pub async fn register(&self) -> Result<(), AppError> {
        info!("Registering with icebreakers api...");

        let url = format!("{}/register", self.base_url);
        let body = json!({
            "secret": self.secret,
            "mode": self.mode,
            "port": self.port,
            "reward_address": self.reward_address,
        });

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Registration(format!("Registering failed: {}", e)))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_response = response.json::<ErrorResponse>().await.map_err(|e| {
                AppError::Registration(format!("Failed to parse error response: {}", e))
            })?;

            Err(AppError::Registration(format!(
                "Failed to register with Icebreakers API: {} details: {}",
                error_response.reason, error_response.details
            )))
        }
    }
}
