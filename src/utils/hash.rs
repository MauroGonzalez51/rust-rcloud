use anyhow::Context;
use sha2::{Digest, Sha256};
use std::path::Path;

pub struct Hash;

impl Hash {
    fn hash_file(path: &Path) -> anyhow::Result<String> {
        let content =
            std::fs::read(path).with_context(|| format!("failed to read file: {:?}", path))?;

        let mut hasher = Sha256::new();
        hasher.update(&content);

        Ok(format!("{:x}", hasher.finalize()))
    }

    fn hash_directory(path: &Path) -> anyhow::Result<String> {
        let mut file_hashes = vec![];

        for entry in walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let file_hash = Hash::hash_file(entry.path())
                .with_context(|| format!("failed to hash file: {:?}", entry.path()))?;

            file_hashes.push(file_hash);
        }

        let mut hasher = Sha256::new();
        for hash in file_hashes {
            hasher.update(hash.as_bytes());
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    pub fn hash_path(path: &Path) -> anyhow::Result<String> {
        if !path.exists() {
            anyhow::bail!("path does not exists: {:?}", path);
        }

        match path.is_dir() {
            true => Hash::hash_directory(path),
            false => Hash::hash_file(path),
        }
    }

    pub fn hash_bytes(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }
}
