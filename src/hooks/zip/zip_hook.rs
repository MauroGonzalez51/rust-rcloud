use crate::{
    config::prelude::{AppConfig, HookExecType},
    hooks::prelude::{HookContext, HookContextMetadata, ZipHook},
    log_debug, log_info, utils,
};
use anyhow::Context;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

const ZIP_PREFIX: &str = "rcloud-zip-";
const ZIP_SUFFIX: &str = ".zip";
const EXTRACT_PREFIX: &str = "rcloud-extract-";

impl ZipHook {
    fn build_exclude_set(&self) -> anyhow::Result<Option<globset::GlobSet>> {
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

    fn process_directory(
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

    fn process_file(
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

    fn compress(&self, path: &PathBuf, cfg: &AppConfig) -> anyhow::Result<(PathBuf, String)> {
        log_debug!("processing file: {:?}", path);
        if let Some(level) = self.level {
            log_info!("using compression level: {}", level);
        }

        let mut buffer = Vec::<u8>::new();
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buffer));

        let options: zip::write::FileOptions<'_, ()> = zip::write::FileOptions::default()
            .compression_level(self.level)
            .compression_method(zip::CompressionMethod::Zstd);

        let exclude_set = self
            .build_exclude_set()
            .context("failed to build exclude set")?;

        match path.is_dir() {
            true => self
                .process_directory(path, &mut zip, options, exclude_set.as_ref())
                .context("failed to process directory")?,
            false => self
                .process_file(path, &mut zip, options)
                .context("failed to process file")?,
        }

        let cursor = zip.finish().context("failed to finish zip")?;
        let zip_bytes = cursor.into_inner();

        let checksum = utils::Hash::hash_bytes(zip_bytes);

        let mut temp_file = match utils::Directories::tempdir(cfg.core.temp_path.clone())? {
            Some(directory) => tempfile::Builder::new()
                .prefix(ZIP_PREFIX)
                .suffix(ZIP_SUFFIX)
                .tempfile_in(&directory)
                .with_context(|| format!("failed to create temp file in {}", directory.display()))?,
            None => tempfile::NamedTempFile::new()
                .context("failed to create temp file")?,
        };

        temp_file
            .write_all(zip_bytes)
            .context("failed to write zip bytes to temp file")?;

        let (_, file_path) = temp_file.keep().context("failed to persist temp file")?;

        Ok((file_path, checksum))
    }

    fn decompress(&self, path: &PathBuf, cfg: &AppConfig) -> anyhow::Result<PathBuf> {
        let file = std::fs::File::open(path).context("failed to open zip file")?;
        let mut archive = zip::read::ZipArchive::new(file).context("failed to read zip archive")?;

        let temp_dir = match utils::Directories::tempdir(cfg.core.temp_path.clone())? {
            Some(directory) => tempfile::Builder::new()
                .prefix(EXTRACT_PREFIX)
                .tempdir_in(directory)
                .context("failed to create temp dir in custom path")?,
            None => tempfile::tempdir().context("failed to create system temp dir")?,
        };

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).context("failed to get file in zip")?;
            let output_path = temp_dir.path().join(file.name());

            if file.is_dir() {
                std::fs::create_dir_all(&output_path).context("faile to create dirs")?;
                continue;
            }

            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent).context("failed to create dirs")?;
            }

            let mut output_file =
                std::fs::File::create(&output_path).context("failed to create output_file")?;

            std::io::copy(&mut file, &mut output_file).context("failed to copy contents")?;
        }

        Ok(temp_dir.keep())
    }

    pub fn process_path(&self, ctx: &HookContext, cfg: &AppConfig) -> anyhow::Result<HookContext> {
        match self.exec {
            HookExecType::Push => {
                let (path, checksum) = self.compress(&ctx.path, cfg)?;

                Ok(ctx
                    .with_path(path)
                    .with_metadata(HookContextMetadata::ZipChecksum, checksum))
            }
            HookExecType::Pull => {
                let path = self.decompress(&ctx.path, cfg)?;

                Ok(ctx.with_path(path))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::prelude::HookExecType;

    fn hook(level: Option<i64>, exclude: Option<Vec<String>>) -> ZipHook {
        ZipHook {
            exec: HookExecType::Push,
            level,
            exclude,
        }
    }

    #[test]
    fn exclude_set_none_when_empty_or_unset() {
        assert!(hook(None, None).build_exclude_set().unwrap().is_none());
        assert!(
            hook(None, Some(vec![]))
                .build_exclude_set()
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn exclude_set_built_for_valid_globs() {
        let set = hook(None, Some(vec!["*.log".into(), "tmp/**".into()]))
            .build_exclude_set()
            .unwrap();
        let set = set.expect("expected a glob set");
        assert!(set.is_match("error.log"));
        assert!(set.is_match("tmp/x/y.txt"));
        assert!(!set.is_match("keep.txt"));
    }

    #[test]
    fn exclude_set_invalid_glob_errors() {
        // An unclosed character class is an invalid glob.
        let result = hook(None, Some(vec!["[".into()])).build_exclude_set();
        assert!(result.is_err());
    }

    #[test]
    fn compress_then_decompress_round_trips_a_file() -> anyhow::Result<()> {
        let cfg = AppConfig::default();
        let dir = tempfile::tempdir()?;
        let src = dir.path().join("data.txt");
        fs::write(&src, b"zip me up")?;

        let h = hook(Some(6), None);
        let (archive, checksum) = h.compress(&src, &cfg)?;
        assert!(archive.exists());
        assert!(!checksum.is_empty());

        let extracted_dir = h.decompress(&archive, &cfg)?;
        let extracted = extracted_dir.join("data.txt");
        assert!(extracted.exists());
        assert_eq!(fs::read(&extracted)?, b"zip me up");
        Ok(())
    }

    #[test]
    fn compress_respects_exclusions() -> anyhow::Result<()> {
        let cfg = AppConfig::default();
        let dir = tempfile::tempdir()?;
        let src = dir.path().join("payload");
        fs::create_dir(&src)?;
        fs::write(src.join("keep.txt"), b"keep")?;
        fs::write(src.join("drop.log"), b"drop")?;

        let h = hook(Some(6), Some(vec!["*.log".into()]));
        let (archive, _) = h.compress(&src, &cfg)?;

        let extracted = h.decompress(&archive, &cfg)?;
        assert!(extracted.join("keep.txt").exists());
        assert!(!extracted.join("drop.log").exists());
        Ok(())
    }
}
