use serde::{Deserialize, Serialize};

/// A configured rclone remote.
///
/// `remote_name` must match the name of a remote already configured in
/// `rclone` itself; rcloud only records the reference.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Remote {
    /// Stable unique id (UUID v4).
    pub id: String,
    /// Name of the rclone remote (as known to `rclone`).
    pub remote_name: String,
    /// Provider label (e.g. "drive", "s3"), informational.
    pub provider: String,
}
