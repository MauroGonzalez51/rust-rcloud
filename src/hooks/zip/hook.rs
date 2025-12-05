use crate::{
    config::prelude::{AppConfig, Hook, HookExecType},
    define_hook,
    hooks::prelude::{HookContext, HookContextMetadata},
    log_debug, log_info, utils,
};
use anyhow::Context;
use std::io::Write;

define_hook!(ZipHook {
    level: Option<i64>,
    exclude: Option<Vec<String>>,
});

impl Hook for ZipHook {
    fn process(&self, ctx: HookContext, cfg: &AppConfig) -> anyhow::Result<HookContext> {
        if !ctx.file_exists() {
            anyhow::bail!("source file does not exist: {:?}", &ctx.path);
        }

        let base_temp_dir = || -> anyhow::Result<Option<std::path::PathBuf>> {
            if let Some(path) = &cfg.core.temp_path {
                if !path.exists() {
                    std::fs::create_dir_all(path).with_context(|| {
                        format!("failed to create custom temp directory: {}", path.display())
                    })?;
                }

                return Ok(Some(path.clone()));
            }

            Ok(None)
        };

        match self.exec {
            HookExecType::Push => {
                let path = &ctx.path;

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

                let checksum = utils::hash::Hash::hash_bytes(zip_bytes);

                let mut temp_file = match base_temp_dir()? {
                    Some(directory) => tempfile::Builder::new()
                        .prefix("rcloud-zip-")
                        .suffix(".zip")
                        .tempfile_in(directory)
                        .context("failed to create temp file in custom directory")?,
                    None => tempfile::NamedTempFile::new()
                        .context("failed to create temp file in system directory")?,
                };

                temp_file
                    .write_all(zip_bytes)
                    .context("failed to write zip bytes to temp file")?;

                let (_, file_path) = temp_file.keep().context("failed to persist temp file")?;

                Ok(HookContext::new(
                    file_path,
                    &ctx.rclone_path,
                    &ctx.remote_config,
                    &ctx.path_config,
                )
                .with_metadata(HookContextMetadata::ZipChecksum, checksum))
            }

            HookExecType::Pull => {
                let file = std::fs::File::open(&ctx.path).context("failed to open zip file")?;
                let mut archive =
                    zip::read::ZipArchive::new(file).context("failed to read zip archive")?;

                let temp_dir = match base_temp_dir()? {
                    Some(directory) => tempfile::Builder::new()
                        .prefix("rcloud-extract-")
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

                    let mut output_file = std::fs::File::create(&output_path)
                        .context("failed to create output_file")?;

                    std::io::copy(&mut file, &mut output_file)
                        .context("failed to copy contents")?;
                }

                Ok(HookContext::new(
                    temp_dir.keep(),
                    &ctx.rclone_path,
                    &ctx.remote_config,
                    &ctx.path_config,
                ))
            }
        }
    }
}
