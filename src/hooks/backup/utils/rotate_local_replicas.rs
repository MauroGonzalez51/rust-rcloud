use crate::hooks::backup::backup_hook::BackupHookReplica;
use anyhow::Context;

pub fn rotate_local_replicas(
    local_replicas: &mut [BackupHookReplica],
    max_replicas: usize,
) -> anyhow::Result<()> {
    local_replicas.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    let current_count = local_replicas.len();

    if current_count >= max_replicas {
        let to_remove = (current_count - max_replicas) + 1;

        for old in local_replicas.iter().rev().take(to_remove) {
            if old.path.is_dir() {
                std::fs::remove_dir_all(&old.path)
                    .with_context(|| format!("failed to remove old replica {:?}", old.path))?;

                continue;
            }

            if old.path.is_file() {
                std::fs::remove_file(&old.path)
                    .with_context(|| format!("failed to remove old replica: {:?}", old.path))?;

                continue;
            }
        }
    }

    Ok(())
}
