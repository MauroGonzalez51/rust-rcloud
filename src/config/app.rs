use crate::log_info;
use anyhow::Context;
use rust_embed::Embed;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Embed)]
#[folder = "assets/"]
struct Asset;

/// Application-wide settings loaded from `rcloud.toml`.
///
/// If the file is missing, a default is materialized from the embedded
/// `assets/default_config.toml`.
#[derive(Debug, Deserialize, Default, Clone)]
pub struct AppConfig {
    /// Core behavior (temp directory, ...).
    pub core: CoreConfig,

    /// TUI settings.
    #[serde(default)]
    pub tui: TuiConfig,
}

/// Core, non-UI settings.
#[derive(Debug, Deserialize, Default, Clone)]
pub struct CoreConfig {
    /// Directory for intermediate files created by hooks. Falls back to the
    /// system temp dir when unset.
    pub temp_path: Option<PathBuf>,
}

/// Terminal UI configuration.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct TuiConfig {
    /// Key bindings for navigation.
    #[serde(default)]
    pub keys: KeyBindings,
}

/// Navigation key bindings for the TUI (vim-style defaults).
#[derive(Debug, Deserialize, Clone)]
pub struct KeyBindings {
    /// Quit the TUI.
    pub quit: Vec<char>,
    /// Move selection up.
    pub up: Vec<char>,
    /// Move selection down.
    pub down: Vec<char>,
    /// Go back / to parent.
    pub left: Vec<char>,
    /// Enter submenu / execute.
    pub right: Vec<char>,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            quit: vec!['q'],
            up: vec!['k'],
            down: vec!['j'],
            left: vec!['h'],
            right: vec!['l'],
        }
    }
}

impl AppConfig {
    pub fn load(path: &PathBuf) -> anyhow::Result<Self> {
        if !path.exists() {
            Self::create_default_config(path)?;
        }

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read config file: {}", path.display()))?;

        let config = toml::from_str::<AppConfig>(&content)
            .with_context(|| format!("failed to parse config file: {}", path.display()))?;

        Ok(config)
    }

    fn create_default_config(path: &PathBuf) -> anyhow::Result<()> {
        if let Some(parent) = path.parent()
            && !parent.exists()
        {
            std::fs::create_dir_all(parent).with_context(|| {
                format!(
                    "failed to create parent directory for path: {}",
                    path.display()
                )
            })?;
        }

        let file = Asset::get("default_config.toml").context("default_config does not exists")?;

        let content = std::str::from_utf8(file.data.as_ref())
            .context("failed to get default_config content")?;

        std::fs::write(path, content)
            .with_context(|| format!("failed to create default config at: {}", path.display()))?;

        log_info!("created default configuration file at: {}", path.display());

        Ok(())
    }
}
