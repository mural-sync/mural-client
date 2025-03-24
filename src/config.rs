use std::path::{Path, PathBuf};

use crate::prelude::*;

const TARGET: &str = "mural_client::config";

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct Config {
    server_url: String,
    pool_name: String,
}

impl Config {
    pub fn load(custom_config_dir: Option<&PathBuf>) -> Result<Self> {
        let config_home_path = custom_config_dir
            .map(|custom_config_dir| custom_config_dir.to_path_buf())
            .unwrap_or(Self::config_home_path()?);
        let _ = std::fs::create_dir_all(&config_home_path);
        let config_file_path = config_home_path.join("config.toml");

        info!(target: TARGET, "loading configuration from '{}'", config_file_path.display());
        let config_file_content = match std::fs::read_to_string(config_file_path) {
            Ok(config_file_content) => config_file_content,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                info!("config file does not exist; using default configuration");
                "".to_string()
            }
            Err(e) => return Err(Error::ConfigRead(e)),
        };

        Ok(toml::from_str(&config_file_content)?)
    }

    fn config_home_path() -> Result<PathBuf> {
        std::env::var("MURAL_CLIENT_CONFIG_HOME")
            .map(|raw_file_path| Path::new(&raw_file_path).to_path_buf())
            .or(
                directories::ProjectDirs::from("ch", "Mural Sync", "Mural Client")
                    .map(|project_dirs| project_dirs.config_local_dir().to_path_buf())
                    .ok_or(Error::ConfigHome),
            )
    }

    pub fn server_url(&self) -> &String {
        &self.server_url
    }

    pub fn pool_name(&self) -> &String {
        &self.pool_name
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_url: "http://localhost:46666".to_string(),
            pool_name: "default".to_string(),
        }
    }
}
