use crate::config::prelude::*;
use serde::{Deserialize, Serialize};

/// A single local <-> remote sync mapping plus its hook pipeline.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PathConfig {
    /// Stable unique id (UUID v4).
    pub id: String,

    /// Id of the [`Remote`] this path syncs against.
    pub remote_id: String,
    /// Absolute local path.
    pub local_path: String,
    /// Path on the remote.
    pub remote_path: String,

    /// Content hash of the last successful sync, used to skip unchanged syncs.
    #[serde(default)]
    pub hash: Option<String>,

    /// Remote filename produced by the push pipeline (base name plus any
    /// hook-added extensions). Recorded on push so pull looks for the exact
    /// same name instead of recomputing it from the pull hook order. `None`
    /// for paths that have never been pushed.
    #[serde(default)]
    pub remote_filename: Option<String>,

    /// Tags for batch operations (`sync all --tags`).
    #[serde(default)]
    pub tags: Vec<String>,

    /// Hooks applied on push and pull.
    pub hooks: PathConfigHooks,
}

/// Ordered hook pipelines for each sync direction.
///
/// On push, `push` hooks run in order before upload. On pull, `pull` hooks run
/// in reverse order after download.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PathConfigHooks {
    /// Hooks applied before uploading.
    #[serde(default)]
    pub push: Vec<HookConfig>,

    /// Hooks applied after downloading.
    #[serde(default)]
    pub pull: Vec<HookConfig>,
}
