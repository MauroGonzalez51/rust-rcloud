use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{self, Read};
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
            .truncate(true)
            .create(true)
            .open(config_path)
        {
            Err(_) => {
                return Err(ConfigError::Custom(
                    "[ Error ] Failed to open file".to_string(),
                ));
            }
            Ok(file) => file,
        };

        if let Err(err) = file.lock_exclusive() {
            return Err(ConfigError::Io(err));
        }

        let mut contents = String::new();
        if let Err(err) = file.read_to_string(&mut contents) {
            return Err(ConfigError::Io(err));
        }

        if let Err(err) = fs2::FileExt::unlock(&file) {
            return Err(ConfigError::Io(err));
        }

        if contents.is_empty() {
            return Ok(Config {
                config_path: config_path.into(),
            });
        }

        match serde_json::from_str::<Config>(&contents) {
            Ok(mut loaded) => {
                loaded.config_path = config_path.into();
                Ok(loaded)
            }
            Err(err) => Err(ConfigError::Serde(err)),
        }
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
