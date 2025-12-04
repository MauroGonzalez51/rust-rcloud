use crate::config::prelude::{PathConfig, Remote};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookContextMetadata {
    SourceLocalPath,
    SourceRemotePath,
    ZipChecksum,
    CalculatedHash,
}

#[derive(Debug, Clone)]
pub struct HookContext {
    pub path: PathBuf,
    pub rclone_path: String,
    pub remote_config: Remote,
    pub path_config: PathConfig,
    pub metadata: std::collections::HashMap<HookContextMetadata, String>,
}

impl HookContext {
    pub fn new(
        path: PathBuf,
        rclone_path: &str,
        remote_config: &Remote,
        path_config: &PathConfig,
    ) -> Self {
        Self {
            path,
            metadata: std::collections::HashMap::new(),
            rclone_path: rclone_path.to_string(),
            remote_config: remote_config.clone(),
            path_config: path_config.clone(),
        }
    }

    pub fn with_metadata(mut self, key: HookContextMetadata, value: impl Into<String>) -> Self {
        self.metadata.insert(key, value.into());
        self
    }

    pub fn file_exists(&self) -> bool {
        self.path.exists()
    }
}
