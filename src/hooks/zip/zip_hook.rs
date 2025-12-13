use crate::{config::app::AppConfig, hooks::zip::ZipHook, log_info};
use anyhow::Context;
use std::{fs, io::Write, path::Path};

impl ZipHook {
    pub fn build_exclude_set(&self) -> anyhow::Result<Option<globset::GlobSet>> {
        match &self.exclude {
            Some(patterns) if !patterns.is_empty() => {
                let mut builder = globset::GlobSetBuilder::new();

                for pattern in patterns {
                    builder.add(
                        globset::Glob::new(pattern)
                            .with_context(|| format!("invalid glob pattern: {}", pattern))?,
                    );
                }

                Ok(Some(builder.build().context("failed to build glob set")?))
            }
            _ => Ok(None),
        }
    }

    pub fn process_directory(
        &self,
        path: &Path,
        zip: &mut zip::ZipWriter<std::io::Cursor<&mut Vec<u8>>>,
        options: zip::write::FileOptions<'_, ()>,
        exclude_set: Option<&globset::GlobSet>,
    ) -> anyhow::Result<()> {
        for entry in walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let relative_path = entry
                .path()
                .strip_prefix(path)
                .context("failed to build relative path")?;

            if let Some(set) = exclude_set
                && set.is_match(relative_path)
            {
                log_info!("excluding: {}", relative_path.display());
                continue;
            }

            let file_content = fs::read(entry.path())
                .with_context(|| format!("failed to read file: {:?}", entry.path()))?;

            let zip_path = relative_path
                .components()
                .filter_map(|c| c.as_os_str().to_str())
                .collect::<Vec<_>>()
                .join("/");

            zip.start_file(&zip_path, options)
                .with_context(|| format!("failed to add file to zip: {}", zip_path))?;

            zip.write_all(&file_content)
                .context("failed to write file to zip")?;

            log_info!("added: {} ({} bytes)", zip_path, file_content.len());
        }

        Ok(())
    }

    pub fn process_file(
        &self,
        path: &Path,
        zip: &mut zip::ZipWriter<std::io::Cursor<&mut Vec<u8>>>,
        options: zip::write::FileOptions<'_, ()>,
    ) -> anyhow::Result<()> {
        let file_content =
            fs::read(path).with_context(|| format!("failed to read file: {:?}", path))?;

        let file_name = path
            .file_name()
            .or_else(|| path.file_name())
            .map(|n| n.to_string_lossy())
            .ok_or_else(|| anyhow::anyhow!("failed to determine file name"))?;

        zip.start_file(&file_name, options)
            .context("failed to start file in zip")?;

        zip.write_all(&file_content)
            .context("failed to write file to zip")?;

        log_info!("added: {:?} ({} bytes)", file_name, file_content.len());

        Ok(())
    }

    pub fn base_temp_dir(cfg: &AppConfig) -> anyhow::Result<Option<std::path::PathBuf>> {
        if let Some(path) = &cfg.core.temp_path {
            if !path.exists() {
                std::fs::create_dir_all(path).with_context(|| {
                    format!("failed to create custom temp directory: {}", path.display())
                })?;
            }

            return Ok(Some(path.clone()));
        }

        Ok(None)
    }
}
