use serde::{Deserialize, Serialize};
use std::path::Path;

/// Extra Malachite configuration options
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_engine_url")]
    pub engine_url: String,
    #[serde(default = "default_eth_url")]
    pub eth_url: String,
    #[serde(default = "default_wt_path")]
    pub wt_path: String,
}

fn default_engine_url() -> String {
    "http://localhost:8551".to_string()
}

fn default_eth_url() -> String {
    "http://localhost:8545".to_string()
}

fn default_wt_path() -> String {
    "./assets/jwtsecret".to_string()
}

/// load_config parses the environment variables and loads the provided config file path
/// to create a Config struct.
pub fn load_config(config_file_path: &Path, prefix: Option<&str>) -> Result<AppConfig, String> {
    config::Config::builder()
        .add_source(config::File::from(config_file_path))
        .add_source(config::Environment::with_prefix(prefix.unwrap_or("MALACHITE")).separator("__"))
        .build()
        .map_err(|error| error.to_string())?
        .try_deserialize()
        .map_err(|error| error.to_string())
}