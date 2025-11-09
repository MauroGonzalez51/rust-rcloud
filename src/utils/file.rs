use anyhow::Context;
use std::{io::Write, path::PathBuf};

pub trait TempFileWriter {
    fn write_temp(&self) -> anyhow::Result<PathBuf>;
}

impl TempFileWriter for [u8] {
    fn write_temp(&self) -> anyhow::Result<PathBuf> {
        let mut temp_file = tempfile::NamedTempFile::new().context("failed to create temp file")?;

        temp_file
            .write_all(self)
            .context("failed to write to temp_file")?;

        let (_, path) = temp_file.keep().context("failed to persist temp file")?;

        Ok(path)
    }
}
