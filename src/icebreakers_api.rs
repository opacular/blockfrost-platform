use crate::{cli::Config, errors::AppError};
use reqwest::Client;
use serde_json::json;
use tracing::{error, info};

pub struct IcebreakersAPI {
    client: Client,
    base_url: String,
    secret: String,
    mode: String,
    reward_address: String,
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
            reward_address: config.reward_address.clone(),
        };

        // Register with Icebreakers API
        if let Err(e) = icebreakers_api.register().await {
            error!("Failed to register with Icebreakers API: {}", e);
            return Err(e);
        } else {
            // info!("Successfully registered with Icebreakers API.");
        }

        Ok(icebreakers_api)
    }

    /// Registers with the Icebreakers API
    pub async fn register(&self) -> Result<(), AppError> {
        info!("Skipping registering with Icebreakers API...");

        let url = format!("{}/register", self.base_url);
        let body = json!({
            "secret": self.secret,
            "mode": self.mode,
            "port": 3001,
            "reward_address": self.reward_address,
        });

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::RegistrationError(format!("Request failed: {}", e)))?;

        if response.status().is_success() {
            Ok(())
        } else {
            // let status = response.status();
            // let error_text = response.text().await.unwrap_or_default();

            Ok(())
            // Err(AppError::RegistrationError(format!(
            //     "Failed with status {}: {}",
            //     status, error_text
            // )))
        }
    }
}
