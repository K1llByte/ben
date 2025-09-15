use std::{default::Default, io, path::Path};

use serde::Deserialize;
use thiserror::Error;

#[derive(Default, Deserialize)]
pub struct Config {
    pub discord_token: String,
    pub cmc_api_key: String,
    #[serde(default)]
    pub use_cmc_sandbox_api: bool,
    #[serde(default)]
    pub bot_admins: Vec<u64>,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("{0}")]
    Io(#[from] io::Error),

    #[error("{0}")]
    Toml(#[from] toml::de::Error),
}

type ConfigResult<T> = Result<T, ConfigError>;

impl Config {
    pub async fn from_file<P: AsRef<Path>>(path: P) -> ConfigResult<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }
}
