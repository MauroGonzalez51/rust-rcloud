use crate::{config::prelude::HookContext, hooks::backup::BackupHook};
use anyhow::Context;

#[derive(Debug)]
struct ReplicaInfo {
    path: std::path::PathBuf,
    timestamp: u64,
    replica_number: u32,
}

impl BackupHook {
    fn parse_replica(path: &std::path::Path, re: &regex::Regex) -> anyhow::Result<ReplicaInfo> {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .context("failed to get file name")?;

        let caps = re.captures(file_name).context("failed to get captures")?;

        let timestamp = caps
            .get(1)
            .map(|n| n.as_str())
            .context("failed to get timestamp")?
            .parse::<u64>()
            .context("failed to parse timestamp")?;

        let replica_number = caps
            .get(2)
            .map(|n| n.as_str())
            .context("failed to get replica number")?
            .parse::<u32>()
            .context("failed to parse replica number")?;

        Ok(ReplicaInfo {
            path: path.to_path_buf(),
            timestamp,
            replica_number,
        })
    }

    fn get_local_replicas(&self) -> anyhow::Result<Vec<ReplicaInfo>> {
        let directory = std::path::Path::new(
            self.local_path
                .as_ref()
                .expect("local path must be declared"),
        );

        let mut replicas = Vec::new();
        let re = regex::Regex::new(r"^(\d+)\.(\d+)$").context("failed to create regex")?;

        if directory.exists() && directory.is_dir() {
            for entry in std::fs::read_dir(directory)
                .with_context(|| format!("failed to read directory: {:?}", directory))?
            {
                let entry = entry.context("failed to get directory entry")?;
                let path = entry.path();

                if let Ok(replica_info) = Self::parse_replica(&path, &re) {
                    replicas.push(replica_info)
                }
            }
        }

        Ok(replicas)
    }

    fn rotate_local_replicas(&self, local_replicas: &mut [ReplicaInfo]) -> anyhow::Result<()> {
        local_replicas.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        let max_replicas = self.replicas as usize;
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

    fn create_local_backup(&self, ctx: &HookContext, replica_number: u32) -> anyhow::Result<()> {
        let directory = std::path::Path::new(
            self.local_path
                .as_ref()
                .expect("local path must be declared"),
        );

        if !directory.exists() {
            std::fs::create_dir_all(directory)
                .with_context(|| format!("failed to create backup directory: {:?}", directory))?;
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .context("failed to get timestap")?
            .as_secs();

        let backup_path = directory.join(format!("{}.{}", timestamp, replica_number));

        if ctx.path.is_file() {
            std::fs::copy(&ctx.path, &backup_path)
                .with_context(|| format!("failed to copy file to backup: {:?}", backup_path))?;

            return Ok(());
        }

        if ctx.path.is_dir() {
            std::fs::create_dir_all(&backup_path)
                .with_context(|| format!("failed to create backup directory: {:?}", backup_path))?;

            fs_extra::dir::copy(&ctx.path, &backup_path, &fs_extra::dir::CopyOptions::new())
                .with_context(|| {
                    format!("failed to copy directory to backup: {:?}", backup_path)
                })?;

            return Ok(());
        }

        Ok(())
    }

    pub fn backup_local(&self, ctx: &HookContext) -> anyhow::Result<()> {
        if self.local_path.is_none() {
            anyhow::bail!("local path must be declared in order to perform a local backup");
        }

        let mut local_replicas = self
            .get_local_replicas()
            .context("failed to get local replicas")?;

        self.rotate_local_replicas(&mut local_replicas)
            .context("failed to rotate local replicas")?;

        let next_replica_number = local_replicas
            .first()
            .map(|r| r.replica_number + 1)
            .unwrap_or(1);

        self.create_local_backup(ctx, next_replica_number)
            .context("failed to create local backup")?;

        Ok(())
    }
}
