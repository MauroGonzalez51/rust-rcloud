use crate::{config::prelude::Remote, hooks::backup::backup_hook::BackupHookReplica, log_warn};
use anyhow::Context;

pub fn rotate_remote_replicas(
    remote_replicas: &mut [BackupHookReplica],
    max_replicas: usize,
    rclone_path: &str,
    remote_config: &Remote,
    remote_backup_path: &str,
) -> anyhow::Result<()> {
    remote_replicas.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    let current_count = remote_replicas.len();

    if current_count >= max_replicas {
        let to_remove = (current_count - max_replicas) + 1;

        for old in remote_replicas.iter().rev().take(to_remove) {
            let filename = old
                .path
                .file_name()
                .unwrap_or(old.path.as_os_str())
                .to_string_lossy();

            let remote_path = format!(
                "{}:{}/{}",
                remote_config.remote_name, remote_backup_path, filename
            );

            let output = std::process::Command::new(rclone_path)
                .args(["purge", &remote_path])
                .output()
                .with_context(|| format!("failed to purge remote file/dir: {}", remote_path,))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                log_warn!(
                    "failed to purge remote file/dir: {} ({})",
                    remote_path,
                    stderr
                )
            }
        }
    }

    Ok(())
}
