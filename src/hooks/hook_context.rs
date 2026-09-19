use crate::config::prelude::{PathConfig, Remote};
use std::path::PathBuf;

/// Keys for values passed between hooks (and between a hook and the sync
/// engine) through [`HookContext::metadata`].
///
/// The metadata map is the side channel hooks use to communicate: for example
/// the Zip hook records a [`ZipChecksum`](HookContextMetadata::ZipChecksum)
/// and the push engine seeds a
/// [`CalculatedHash`](HookContextMetadata::CalculatedHash) used for skip
/// detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookContextMetadata {
    /// Original local path of the source (set on pull).
    SourceLocalPath,
    /// Original remote path of the source (set on pull).
    SourceRemotePath,
    /// SHA-256 checksum of the produced zip archive.
    ZipChecksum,
    /// Content hash of the source used to skip unchanged syncs.
    CalculatedHash,
}

/// Mutable state threaded through the hook pipeline for one sync operation.
///
/// `path` is the value each hook reads and rewrites: it starts at the source
/// (local path on push, downloaded temp file on pull) and is replaced by every
/// hook that transforms the content. The remaining fields are read-only
/// context (which remote, which path config, where the `rclone` binary lives).
/// `metadata` is a free-form map for cross-hook communication keyed by
/// [`HookContextMetadata`].
#[derive(Debug, Clone)]
pub struct HookContext {
    /// Current working path. Rewritten by each transforming hook.
    pub path: PathBuf,
    /// Path to the `rclone` executable (needed by hooks that call rclone).
    pub rclone_path: String,
    /// Remote this sync targets.
    pub remote_config: Remote,
    /// Path configuration driving this sync.
    pub path_config: PathConfig,
    /// Cross-hook communication channel keyed by [`HookContextMetadata`].
    pub metadata: std::collections::HashMap<HookContextMetadata, String>,
}

impl HookContext {
    /// Creates a context rooted at `path` for the given remote/path config.
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

    /// Returns a copy with `key` set to `value` in the metadata map.
    pub fn with_metadata(mut self, key: HookContextMetadata, value: impl Into<String>) -> Self {
        self.metadata.insert(key, value.into());
        self
    }

    /// Returns a copy with `path` replaced, preserving all other fields and
    /// metadata. This is how hooks hand off their transformed output.
    pub fn with_path(&self, path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            metadata: self.metadata.clone(),
            rclone_path: self.rclone_path.clone(),
            remote_config: self.remote_config.clone(),
            path_config: self.path_config.clone(),
        }
    }

    /// Whether the current `path` exists on disk.
    pub fn file_exists(&self) -> bool {
        self.path.exists()
    }
}
