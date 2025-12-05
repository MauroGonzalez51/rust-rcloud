use anyhow::Context;
use directories::ProjectDirs;
use std::path::PathBuf;

pub fn get_default_config_path() -> anyhow::Result<PathBuf> {
    if let Some(project_dirs) = ProjectDirs::from("", "", "rcloud") {
        let config_dir = project_dirs.config_dir();

        if !config_dir.exists() {
            std::fs::create_dir_all(config_dir).context("failed to create config directory")?;
        }

        return Ok(PathBuf::from(config_dir));
    }

    Ok(PathBuf::from("."))
}
