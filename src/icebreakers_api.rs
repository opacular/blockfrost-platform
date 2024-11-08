use crate::{cli::Config, errors::AppError};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::sync::{Arc, RwLock};
use tracing::{info, warn};

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

const API_URL: &str = "https://api-dev.icebreakers.blockfrost.io";
// const API_URL: &str = "http://localhost:3000";
// const API_URL: &str = "https://icebreakers-api.blockfrost.io";

impl IcebreakersAPI {
    /// Creates a new `IcebreakersAPI` instance or logs a warning if not configured
    pub async fn new(config: &Config) -> Result<Option<Arc<RwLock<Self>>>, AppError> {
        match &config.icebreakers_config {
            Some(icebreakers_config) => {
                info!("Connecting to Icebreakers API...");

                let client = Client::new();
                let base_url = API_URL.to_string();

                let icebreakers_api = IcebreakersAPI {
                    client,
                    base_url,
                    secret: icebreakers_config.secret.clone(),
                    mode: config.mode.to_string(),
                    port: config.server_port,
                    reward_address: icebreakers_config.reward_address.clone(),
                };

                icebreakers_api.register().await?;

                info!("Successfully registered with Icebreakers API.");

                Ok(Some(Arc::new(RwLock::new(icebreakers_api))))
            }
            None => {
                // Logging the solitary mode warning
                warn!(" __________________________________________ ");
                warn!("/ Running in solitary mode.                \\");
                warn!("|                                          |");
                warn!("\\ You're not part of the Blockfrost fleet! /");
                warn!(" ------------------------------------------ ");
                warn!("        \\   ^__^");
                warn!("         \\  (oo)\\_______");
                warn!("            (__)\\       )\\/\\");
                warn!("                ||----w |");
                warn!("                ||     ||");

                Ok(None)
            }
        }
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
