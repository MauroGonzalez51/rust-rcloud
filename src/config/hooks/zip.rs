use crate::{config::prelude::*, define_hook, log_info, utils::file::TempFileWriter};
use anyhow::{Context, bail};
use inquire::Text;
use sha2::{Digest, Sha256};
use std::{fs, io::Write, path::Path};

define_hook!(ZipHook {
    source: String,
    level: Option<i64>,
    exclude: Option<Vec<String>>,
});

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
    ) -> anyhow::Result<Vec<u8>> {
        let mut checksums = Vec::<u8>::new();

        for entry in walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let relative_path = entry
                .path()
                .strip_prefix(path)
                .context("failed to build relative path")?;

            if let Some(set) = exclude_set {
                if set.is_match(relative_path) {
                    log_info!("excluding: {:?}", relative_path);
                    continue;
                }
            }

            let file_content = fs::read(entry.path())
                .with_context(|| format!("failed to read file: {:?}", entry.path()))?;

            let mut hasher = Sha256::new();
            hasher.update(&file_content);
            checksums.extend_from_slice(&hasher.finalize());

            zip.start_file(relative_path.to_string_lossy(), options)
                .with_context(|| format!("failed to add file to zip: {:?}", relative_path))?;

            zip.write_all(&file_content)
                .context("faile to write file to zip")?;

            log_info!("added: {:?} ({} bytes)", relative_path, file_content.len());
        }

        Ok(checksums)
    }

    fn process_file(
        &self,
        path: &Path,
        zip: &mut zip::ZipWriter<std::io::Cursor<&mut Vec<u8>>>,
        options: zip::write::FileOptions<'_, ()>,
    ) -> anyhow::Result<Vec<u8>> {
        let file_content =
            fs::read(path).with_context(|| format!("failed to read file: {:?}", path))?;

        let mut hasher = Sha256::new();
        hasher.update(&file_content);

        let checksum = hasher.finalize();

        let file_name = Path::new(&self.source)
            .file_name()
            .or_else(|| path.file_name())
            .map(|n| n.to_string_lossy())
            .ok_or_else(|| anyhow::anyhow!("failed to determine file name"))?;

        zip.start_file(&file_name, options)
            .context("failed to start file in zip")?;

        zip.write_all(&file_content)
            .context("failed to write file to zip")?;

        log_info!("added: {:?} ({} bytes)", file_name, file_content.len());

        Ok(checksum.to_vec())
    }
}

impl Hook for ZipHook {
    fn name(&self) -> &'static str {
        "zip"
    }

    fn exec_type(&self) -> &HookExecType {
        &self.exec
    }

    fn hook_type(&self) -> &Hooks {
        &Hooks::Zip
    }

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
                    .compression_method(zip::CompressionMethod::Deflated);

                let exclude_set = self
                    .build_exclude_set()
                    .context("failed to build exclude set")?;

                let checksums = match path.is_dir() {
                    true => self
                        .process_directory(path, &mut zip, options, exclude_set.as_ref())
                        .context("failed to process directory")?,
                    false => self
                        .process_file(path, &mut zip, options)
                        .context("failed to process file")?,
                };

                let cursor = zip.finish().context("failed to finish zip")?;
                let zip_bytes = cursor.into_inner();

                let mut final_hasher = Sha256::new();
                final_hasher.update(&checksums);
                let final_checksum = format!("{:x}", final_hasher.finalize());

                let file_path = zip_bytes
                    .write_temp()
                    .context("failed to write temp file")?;

                Ok(HookContext::new(file_path).with_metadata("zip_checksum", final_checksum))
            }

            HookExecType::Pull => {
                let file = std::fs::File::open(&ctx.path).context("failed to open zip file")?;
                let mut archive =
                    zip::read::ZipArchive::new(file).context("failed to read zip archive")?;

                let temp_dir = std::env::temp_dir();

                for i in 0..archive.len() {
                    let mut file = archive.by_index(i).context("failed to get file in zip")?;
                    let output_path = temp_dir.join(file.name());

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

                Ok(HookContext::new(temp_dir))
            }
        }
    }
}

impl ZipHookConfig {
    pub fn build(exec_type: HookExecType, source: &str) -> anyhow::Result<HookConfig> {
        log_info!("configuring {} for {}", Hooks::Zip, exec_type);

        match exec_type {
            HookExecType::Push => {
                let level = Text::new("Compression level (0-9):")
                    .with_default("6")
                    .prompt()
                    .context("failed to get compression level")?
                    .parse::<i64>()
                    .context("failed to parse compresion level")?;

                let exclude = Text::new("Exclude patterns: ")
                    .with_help_message("comma-separated, glob only, optional")
                    .prompt_skippable()
                    .context("failed to get exclude patterns")?;

                let exclude = exclude.map(|s| {
                    s.split(',')
                        .map(|p| p.trim().to_string())
                        .filter(|p| !p.is_empty())
                        .collect()
                });

                Ok(HookConfig::Zip(Self {
                    exec: HookExecType::Push,
                    source: source.to_string(),
                    level: Some(level),
                    exclude,
                }))
            }
            HookExecType::Pull => Ok(HookConfig::Zip(Self {
                exec: HookExecType::Pull,
                source: source.to_string(),
                level: None,
                exclude: None,
            })),
        }
    }
}
