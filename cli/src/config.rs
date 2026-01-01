//! Configuration management for SwarmPool CLI

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Provider ENS name
    pub provider_ens: Option<String>,

    /// Wallet address for payouts
    pub wallet: Option<String>,

    /// GPUs available
    pub gpus: Vec<String>,

    /// Models to process
    pub models: Vec<String>,

    /// Pool ENS
    pub pool: String,

    /// IPFS API URL
    pub ipfs_api: String,
}

impl Config {
    pub fn new() -> Self {
        Self {
            provider_ens: None,
            wallet: None,
            gpus: vec![],
            models: vec!["queenbee-spine".to_string()],
            pool: "swarmpool.eth".to_string(),
            ipfs_api: "http://localhost:5001".to_string(),
        }
    }
}

/// Get the config file path
pub fn get_config_path() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("eth", "swarmpool", "swarm-cli")
        .context("Failed to determine config directory")?;

    let config_dir = proj_dirs.config_dir();
    std::fs::create_dir_all(config_dir)?;

    Ok(config_dir.join("config.toml"))
}

/// Load configuration from file
pub fn load_config() -> Result<Config> {
    let path = get_config_path()?;

    if !path.exists() {
        return Ok(Config::new());
    }

    let content = std::fs::read_to_string(&path)
        .context("Failed to read config file")?;

    let config: Config = toml::from_str(&content)
        .context("Failed to parse config file")?;

    Ok(config)
}

/// Save configuration to file
pub fn save_config(config: &Config) -> Result<()> {
    let path = get_config_path()?;

    let content = toml::to_string_pretty(config)
        .context("Failed to serialize config")?;

    std::fs::write(&path, content)
        .context("Failed to write config file")?;

    Ok(())
}
