use anyhow::Context;
use sha2::{Digest, Sha256};
use std::{fs::File, io::Write, path::PathBuf};

pub trait TempFileWriter {
    fn write_temp(&self) -> anyhow::Result<PathBuf>;
}

impl TempFileWriter for [u8] {
    fn write_temp(&self) -> anyhow::Result<PathBuf> {
        let mut temp_path = std::env::temp_dir();
        let uuid = uuid::Uuid::new_v4();

        let mut hasher = Sha256::new();
        hasher.update(self);

        let checksum = format!("{:x}", hasher.finalize());
        let filename = format!("rcloud-{}-{}.tmp", uuid, &checksum[..8]);
        temp_path.push(filename);

        let mut file = File::create(&temp_path).context("failed to create file")?;
        file.write_all(self).context("failed to write files")?;

        Ok(temp_path)
    }
}
