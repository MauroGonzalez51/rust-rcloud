use crate::hooks::backup::backup_hook::BackupHookReplica;
use anyhow::Context;

pub fn rotate_local_replicas(
    local_replicas: &mut [BackupHookReplica],
    max_replicas: usize,
) -> anyhow::Result<()> {
    local_replicas.sort_by_key(|b| std::cmp::Reverse(b.timestamp));

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn replica(dir: &std::path::Path, timestamp: u64, n: u32) -> BackupHookReplica {
        let path = dir.join(format!("{}.{}", timestamp, n));
        fs::write(&path, b"x").unwrap();
        BackupHookReplica {
            path,
            timestamp,
            replica_number: n,
        }
    }

    #[test]
    fn keeps_all_when_below_max() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let mut replicas = vec![replica(dir.path(), 100, 1)];

        // max=3, only 1 present -> nothing removed.
        rotate_local_replicas(&mut replicas, 3)?;
        assert!(replicas[0].path.exists());
        Ok(())
    }

    #[test]
    fn prunes_oldest_to_make_room_at_max() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        // At max (3 present, max=3): removes (3-3)+1 = 1 oldest.
        let r_old = replica(dir.path(), 100, 1);
        let r_mid = replica(dir.path(), 200, 2);
        let r_new = replica(dir.path(), 300, 3);
        let mut replicas = vec![r_mid, r_new, r_old];

        rotate_local_replicas(&mut replicas, 3)?;

        // Oldest (timestamp 100) is gone; the two newest remain.
        assert!(!dir.path().join("100.1").exists());
        assert!(dir.path().join("200.2").exists());
        assert!(dir.path().join("300.3").exists());
        Ok(())
    }

    #[test]
    fn prunes_multiple_when_over_max() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        // 4 present, max=2: removes (4-2)+1 = 3 oldest, leaving the newest.
        let mut replicas = vec![
            replica(dir.path(), 100, 1),
            replica(dir.path(), 200, 2),
            replica(dir.path(), 300, 3),
            replica(dir.path(), 400, 4),
        ];

        rotate_local_replicas(&mut replicas, 2)?;

        assert!(!dir.path().join("100.1").exists());
        assert!(!dir.path().join("200.2").exists());
        assert!(!dir.path().join("300.3").exists());
        assert!(dir.path().join("400.4").exists());
        Ok(())
    }
}
