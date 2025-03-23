use tokio::fs;

use crate::types::{Backend, Frontend, ServerSettings};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    #[serde(default)]
    pub server: ServerSettings,
    pub frontends: Vec<Frontend>,
    pub backends: Vec<Backend>,
}

pub async fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let file_path = match std::env::var("CONFIG_FILE") {
        Ok(path) => path,
        Err(_) => "config.yaml".to_string(),
    };

    log::debug!("Loading config from: {}", file_path);

    let yaml_content = fs::read_to_string(file_path).await?;

    let config: Config = serde_yaml::from_str(&yaml_content)?;

    if config.server.enable_https
        && (config.server.cert_path.is_none() || config.server.key_path.is_none())
    {
        return Err("cert_path and key_path must be provided when enable_https is true".into());
    }

    Ok(config)
}
