use crate::{
    config::prelude::{Hook, HookExecType},
    define_hook,
    hooks::prelude::{HookContext, HookContextMetadata},
    log_info,
    utils::{self, file::TempFileWriter},
};
use anyhow::{Context, bail};

define_hook!(ZipHook {
    level: Option<i64>,
    exclude: Option<Vec<String>>,
});

impl Hook for ZipHook {
    fn process(&self, ctx: HookContext) -> anyhow::Result<HookContext> {
        if !ctx.file_exists() {
            bail!("source file does not exist: {:?}", &ctx.path);
        }

        match self.exec {
            HookExecType::Push => {
                let path = &ctx.path;

                log_info!("processing file: {:?}", path);
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

                let file_path = zip_bytes
                    .write_temp()
                    .context("failed to write temp file")?;

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

                let temp_dir = tempfile::tempdir().context("failed to create temp directory")?;

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
