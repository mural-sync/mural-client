use std::path::{Path, PathBuf};

use crate::prelude::*;

const TARGET: &str = "mural_client::config";

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default = "default_server_url")]
    server_url: String,
    #[serde(default = "default_pool_name")]
    pool_name: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_home_path = Self::config_home_path()?;
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
            .or(xdg::BaseDirectories::with_prefix("mural-client")
                .map(|base_dirs| base_dirs.get_config_home()))
            .map_err(|_| Error::ConfigHome)
    }

    pub fn server_url(&self) -> &String {
        &self.server_url
    }

    pub fn pool_name(&self) -> &String {
        &self.pool_name
    }
}

fn default_server_url() -> String {
    "http://localhost:46666".to_string()
}

fn default_pool_name() -> String {
    "default".to_string()
}
