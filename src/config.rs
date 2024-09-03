use clap::Parser;
use pallas_network::miniprotocols::{MAINNET_MAGIC, PREPROD_MAGIC, PREVIEW_MAGIC, SANCHONET_MAGIC};
use serde::{Deserialize, Deserializer};
use std::str::FromStr;
use std::{fs, path::PathBuf};
use thiserror::Error;
use tracing::Level;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, value_name = "FILE")]
    pub config: PathBuf,

    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Config file is invalid: {0}")]
    InvalidConfig(String),

    #[error("Failed to read config file: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Failed to parse config file: {0}")]
    ParseError(#[from] toml::de::Error),
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerInput {
    pub address: String,
    pub log_level: String,
    pub network: String,
    pub relay: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Server {
    pub address: String,
    #[serde(deserialize_with = "deserialize_log_level")]
    pub log_level: Level,
    pub network_magic: u64,
    pub relay: String,
}

fn deserialize_log_level<'de, D>(deserializer: D) -> Result<Level, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Level::from_str(&s.to_lowercase()).map_err(serde::de::Error::custom)
}

#[derive(Debug, Deserialize, Clone)]
pub struct ConfigInput {
    pub server: ServerInput,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: Server,
}

pub fn load_config(path: PathBuf) -> Result<Config, ConfigError> {
    let config_file_content = fs::read_to_string(path).expect("Reading config failed");
    let toml_config: ConfigInput =
        toml::from_str(&config_file_content).expect("Config file is invalid");

    let log_level = match toml_config.server.log_level.to_lowercase().as_str() {
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        "trace" => Level::TRACE,
        _ => Level::INFO,
    };

    let network_magic = match toml_config.server.network.as_str() {
        "mainnet" => MAINNET_MAGIC,
        "preprod" => PREPROD_MAGIC,
        "preview" => PREVIEW_MAGIC,
        "sanchonet" => SANCHONET_MAGIC,
        _ => return Err(ConfigError::InvalidConfig("Invalid network".to_string())),
    };

    Ok(Config {
        server: Server {
            address: toml_config.server.address,
            log_level,
            network_magic,
            relay: toml_config.server.relay,
        },
    })
}
