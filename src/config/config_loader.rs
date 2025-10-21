use std::io::{self, Read};

use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::path::PathBuf;
// use transaction::prelude::*;

// pub type ConfigTx<'a, T> = Box<dyn Transaction<Ctx = Config, Item = T, Err = ConfigError> + 'a>;

#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    Serde(serde_json::Error),
    Custom(String),
}

impl From<io::Error> for ConfigError {
    fn from(value: io::Error) -> Self {
        ConfigError::Io(value)
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(value: serde_json::Error) -> Self {
        ConfigError::Serde(value)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(skip)]
    pub config_path: PathBuf,
}

impl Config {
    pub fn load(config_path: &String) -> Result<Self, ConfigError> {
        let mut file = match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(config_path)
        {
            Err(_) => {
                return Err(ConfigError::Custom(
                    "[ Error ] Failed to open file".to_string(),
                ));
            }
            Ok(file) => file,
        };

        let mut contents = String::new();

        match file.read_to_string(&mut contents) {
            Ok(_) => {}
            Err(error) => return Err(ConfigError::Io(error)),
        };

        if contents.is_empty() {
            return Ok(Config {
                config_path: config_path.into(),
            });
        }

        let loaded: Config = serde_json::from_str(&contents)?;

        Ok(loaded)
    }
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string_pretty(self) {
            Ok(json) => write!(f, "{}", json),
            Err(_) => write!(f, "[ Error ] config could not be serialized"),
        }
    }
}
