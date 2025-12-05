use anyhow::Context;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Default, Clone)]
pub struct AppConfig {
    pub core: Option<CoreConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CoreConfig {
    pub temp_path: Option<PathBuf>,
}

impl AppConfig {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)
            .context("failed to read rcloud.toml configuration file")?;

        let config: AppConfig =
            toml::from_str(&content).context("failed to parse rcloud.toml configuration file")?;

        Ok(config)
    }
}
