use crate::{config::prelude::*, log_debug, log_info, log_warn};
use anyhow::{Context, bail};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;
use thiserror::Error;
use transaction::prelude::*;

#[allow(dead_code)]
type RegistryTx<'a, T> = Box<dyn Transaction<Ctx = Registry, Item = T, Err = RegistryError> + 'a>;

#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("failed to read/write registry file")]
    Io(#[from] std::io::Error),

    #[error("registry file is corrupted or has invalid format")]
    Serde(#[from] serde_json::Error),

    #[allow(dead_code)]
    #[error("{0}")]
    Custom(String),

    #[error("registry is corrupted")]
    Corrupted {
        #[source]
        source: serde_json::Error,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Registry {
    #[serde(skip)]
    pub registry_path: PathBuf,

    #[serde(default)]
    pub remotes: Vec<Remote>,

    #[serde(default)]
    pub paths: Vec<PathConfig>,
}

impl Registry {
    pub fn load(registry_path: &PathBuf) -> anyhow::Result<Self> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(registry_path)
            .context("failed to open registry file")?;

        file.lock_exclusive()
            .context("failed to acquire lock on registry")?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .context("failed to read registry contents")?;

        fs2::FileExt::unlock(&file).context("failed to release lock on registry")?;

        if contents.trim().is_empty() {
            log_warn!(
                "registry file is empty, creating new one at: {}",
                registry_path.display()
            );

            let mut registry = Registry {
                registry_path: registry_path.clone(),
                remotes: vec![],
                paths: vec![],
            };

            registry.save().context("failed to save new registry")?;

            return Ok(registry);
        }

        match serde_json::from_str::<Registry>(&contents) {
            Ok(mut loaded) => {
                loaded.registry_path = registry_path.clone();
                log_debug!("file loaded");
                Ok(loaded)
            }
            Err(err) => {
                bail!(RegistryError::Corrupted { source: err })
            }
        }
    }

    #[allow(dead_code)]
    pub fn tx<F, T>(&mut self, function: F) -> anyhow::Result<()>
    where
        F: FnOnce(&mut Registry) -> T,
    {
        log_debug!("executing transaction in registry file");

        let backup = self.clone();

        function(self);

        match self.save() {
            Ok(_) => {}
            Err(_err) => {
                *self = backup;
            }
        }

        Ok(())
    }

    fn save(&mut self) -> anyhow::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.registry_path)
            .context("failed to open registry file")?;

        file.lock_exclusive()
            .context("failed to acquire lock on registry")?;

        let contents = serde_json::to_string_pretty(&self).context("failed to parse contents")?;

        fs2::FileExt::unlock(&file).context("failed to release lock on registry")?;

        log_info!("saving file");

        file.write_all(contents.as_bytes())
            .context("failed to write contents to file")?;

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
