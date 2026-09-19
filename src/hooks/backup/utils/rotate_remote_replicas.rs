use crate::{
    config::prelude::Remote,
    hooks::{backup::backup_hook::BackupHookReplica, prelude::RcloneRunner},
    log_debug,
};

pub fn rotate_remote_replicas(
    remote_replicas: &mut [BackupHookReplica],
    max_replicas: usize,
    rclone: &dyn RcloneRunner,
    remote_config: &Remote,
    remote_backup_path: &str,
) -> anyhow::Result<()> {
    remote_replicas.sort_by_key(|b| std::cmp::Reverse(b.timestamp));

    let current_count = remote_replicas.len();

    if current_count >= max_replicas {
        let to_remove = (current_count - max_replicas) + 1;

        log_debug!("found {} old replicas to remove", to_remove);

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

            log_debug!("removing old replica: {}", remote_path);

            rclone.purge(&remote_path)?;
        }
    }

    Ok(())
}
