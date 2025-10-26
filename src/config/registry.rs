use super::remote::Remote;
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use transaction::prelude::*;

#[allow(dead_code)]
type RegistryTx<'a, T> = Box<dyn Transaction<Ctx = Registry, Item = T, Err = RegistryError> + 'a>;

#[derive(Debug)]
pub enum RegistryError {
    Io(io::Error),
    Serde(serde_json::Error),
    Custom(String),
}

impl From<io::Error> for RegistryError {
    fn from(value: io::Error) -> Self {
        RegistryError::Io(value)
    }
}

impl From<serde_json::Error> for RegistryError {
    fn from(value: serde_json::Error) -> Self {
        RegistryError::Serde(value)
    }
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::Io(err) => write!(f, "[ ERROR ] (io) {}", err),
            RegistryError::Serde(err) => write!(f, "[ ERROR ] (serde) {}", err),
            RegistryError::Custom(err) => write!(f, "[ ERROR ] {}", err),
        }
    }
}

impl std::error::Error for RegistryError {}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Registry {
    #[serde(skip)]
    pub registry_path: PathBuf,

    #[serde(default)]
    pub remotes: Vec<Remote>,
}

impl Registry {
    pub fn load(registry_path: &PathBuf) -> Result<Self, RegistryError> {
        let mut file = match OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(false)
            .create(true)
            .open(registry_path)
        {
            Err(_) => {
                return Err(RegistryError::Custom(
                    "[ Error ] Failed to open file".to_string(),
                ));
            }
            Ok(file) => file,
        };

        if let Err(err) = file.lock_exclusive() {
            return Err(RegistryError::Io(err));
        }

        let mut contents = String::new();
        if let Err(err) = file.read_to_string(&mut contents) {
            return Err(RegistryError::Io(err));
        }

        if let Err(err) = fs2::FileExt::unlock(&file) {
            return Err(RegistryError::Io(err));
        }

        if contents.is_empty() {
            let mut result = Registry {
                registry_path: registry_path.into(),
                remotes: vec![],
            };

            result.save()?;

            return Ok(result);
        }

        match serde_json::from_str::<Registry>(&contents) {
            Ok(mut loaded) => {
                loaded.registry_path = registry_path.into();
                Ok(loaded)
            }
            Err(err) => Err(RegistryError::Serde(err)),
        }
    }

    #[allow(dead_code)]
    pub fn tx<F, T>(&mut self, function: F) -> Result<T, RegistryError>
    where
        F: FnOnce(&mut Registry) -> T,
    {
        let backup = self.clone();
        let result = function(self);

        match self.save() {
            Ok(_) => Ok(result),
            Err(err) => {
                *self = backup;
                Err(err)
            }
        }
    }

    fn save(&mut self) -> Result<(), RegistryError> {
        let mut file = match OpenOptions::new()
            .write(true)
            .truncate(false)
            .open(&self.registry_path)
        {
            Err(err) => return Err(RegistryError::Io(err)),
            Ok(file) => file,
        };

        if let Err(err) = file.lock_exclusive() {
            return Err(RegistryError::Io(err));
        }

        let contents = match serde_json::to_string_pretty(&self) {
            Ok(json) => json,
            Err(err) => return Err(RegistryError::Serde(err)),
        };

        if let Err(err) = fs2::FileExt::unlock(&file) {
            return Err(RegistryError::Io(err));
        }

        if let Err(err) = file.write_all(contents.as_bytes()) {
            return Err(RegistryError::Io(err));
        }

        Ok(())
    }
}

impl std::fmt::Display for Registry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string_pretty(self) {
            Ok(json) => write!(f, "{}", json),
            Err(_) => write!(f, "[ Error ] config could not be serialized"),
        }
    }
}
