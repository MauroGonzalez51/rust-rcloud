use crate::log_warn;
use anyhow::Context;
use directories::ProjectDirs;
use std::path::PathBuf;

#[derive(Default)]
pub struct Directories {
    pub config_dir: PathBuf,

    #[allow(dead_code)]
    pub data_dir: PathBuf,
}

impl Directories {
    pub fn new() -> anyhow::Result<Self> {
        if let Some(dirs) = ProjectDirs::from("", "", "rcloud") {
            let config_dir = dirs.config_dir().to_path_buf();
            let data_dir = dirs.data_dir().to_path_buf();

            if !config_dir.exists() {
                std::fs::create_dir_all(&config_dir)
                    .context("failed to create config directory")?;
            }

            if !data_dir.exists() {
                std::fs::create_dir_all(&data_dir).context("failed to create data directory")?;
            }

            return Ok(Self {
                config_dir,
                data_dir,
            });
        }

        Ok(Self::default())
    }
}

pub fn directories() -> &'static Directories {
    static DIRS: std::sync::OnceLock<Directories> = std::sync::OnceLock::new();
    DIRS.get_or_init(|| {
        Directories::new().unwrap_or_else(|e| {
            log_warn!("could not resolve directories: {}", e);
            Directories::default()
        })
    })
}
