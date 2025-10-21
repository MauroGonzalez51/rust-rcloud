use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use transaction::prelude::*;

pub type RegistryTx<'a, T> =
    Box<dyn Transaction<Ctx = Registry, Item = T, Err = RegistryError> + 'a>;

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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Registry {
    #[serde(skip)]
    pub config_path: PathBuf,
}

impl Registry {
    pub fn load(config_path: &String) -> Result<Self, RegistryError> {
        let mut file = match OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open(config_path)
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
            return Ok(Registry {
                config_path: config_path.into(),
            });
        }

        match serde_json::from_str::<Registry>(&contents) {
            Ok(mut loaded) => {
                loaded.config_path = config_path.into();
                Ok(loaded)
            }
            Err(err) => Err(RegistryError::Serde(err)),
        }
    }

    pub fn tx<'a, F, T>(self, f: F) -> RegistryTx<'a, T>
    where
        F: Fn(&mut Registry) -> T + 'a,
        T: 'a,
    {
        with_ctx(move |ctx: &mut Registry| {
            let backup = ctx.clone();

            let result = f(ctx);

            if let Err(err) = ctx.save() {
                *ctx = backup;
                return Err(err);
            }

            Ok(result)
        })
        .boxed()
    }

    fn save(&mut self) -> Result<(), RegistryError> {
        let mut file = match OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.config_path)
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
