use crate::{
    config::hook_config::{Hook, HookContext, HookType},
    define_hook, log_error,
    utils::file::TempFileWriter,
};
use anyhow::{Context, bail};
use globset::{Glob, GlobSetBuilder};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::Path,
};

define_hook!(ZipHook {
    source: String,
    level: Option<i64>,
    exclude: Option<Vec<String>>,
});

impl Hook for ZipHook {
    fn name(&self) -> &'static str {
        "zip"
    }

    fn exec_type(&self) -> &HookType {
        &self.exec
    }

    fn process(&self, ctx: HookContext) -> anyhow::Result<HookContext> {
        if !ctx.file_exists() {
            log_error!("file does not exists");
            bail!("file does not exists");
        }

        let path = &ctx.path;
        let mut buffer = Vec::<u8>::new();
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buffer));

        let options: zip::write::FileOptions<'_, ()> = zip::write::FileOptions::default()
            .compression_level(self.level)
            .compression_method(zip::CompressionMethod::Deflated);

        let exclude_set = match self.exclude {
            Some(ref patterns) => {
                let mut builder = GlobSetBuilder::new();
                for pattern in patterns {
                    builder.add(Glob::new(pattern).context("failed to create glob")?);
                }

                Some(builder.build().context("failed to create glob builder")?)
            }
            None => None,
        };

        let mut checksums = Vec::new();

        match path.is_dir() {
            true => {
                for entry in walkdir::WalkDir::new(path)
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter(|e| e.file_type().is_file())
                {
                    let relative_path = entry
                        .path()
                        .strip_prefix(path)
                        .context("failed to build relative path")?;

                    if let Some(ref set) = exclude_set {
                        if set.is_match(relative_path) {
                            continue;
                        }
                    }

                    let mut file = fs::File::open(entry.path()).context("failed to open file")?;
                    let mut file_content = Vec::<u8>::new();

                    file.read_to_end(&mut file_content)
                        .context("failed to read file content")?;

                    let mut hasher = Sha256::new();
                    hasher.update(&file_content);
                    checksums.extend_from_slice(&hasher.finalize());

                    zip.start_file(relative_path.to_string_lossy(), options)
                        .context("failed to start file in zip")?;

                    zip.write_all(&file_content)
                        .context("failed to write file to zip")?;
                }
            }
            false => {
                let mut file = fs::File::open(path).context("failed to open file")?;
                let mut file_content = Vec::<u8>::new();

                file.read_to_end(&mut file_content)
                    .context("failed to read file content")?;

                let mut hasher = Sha256::new();
                hasher.update(&file_content);
                checksums.extend_from_slice(&hasher.finalize());

                let file_name = Path::new(&self.source)
                    .file_name()
                    .map(|n| n.to_string_lossy())
                    .unwrap_or_else(|| path.file_name().unwrap().to_string_lossy());

                zip.start_file(file_name, options)
                    .context("failed to start file in zip")?;

                zip.write_all(&file_content)
                    .context("failed to write file to zip")?;
            }
        };

        let cursor = zip.finish().context("failed to finish zip")?;
        let zip_bytes = cursor.into_inner();

        let mut final_hasher = Sha256::new();
        final_hasher.update(&checksums);

        let final_checksum = format!("{:x}", final_hasher.finalize());

        let file_path = zip_bytes
            .write_temp()
            .context("failed to write temp file")?;

        let metadata = ctx.with_metadata("zip_checksum", final_checksum).metadata;

        Ok(HookContext {
            path: file_path,
            metadata,
        })
    }
}
