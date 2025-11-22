use crate::{
    config::prelude::HookContext,
    hooks::backup::{BackupHook, utils},
};
use anyhow::Context;

#[derive(Debug)]
pub struct BackupHookReplica {
    pub path: std::path::PathBuf,
    pub timestamp: u64,
    pub replica_number: u32,
}

impl BackupHook {
    pub fn backup_local(&self, ctx: &HookContext) -> anyhow::Result<()> {
        if self.local_path.is_none() {
            anyhow::bail!("local path must be declared in order to perform a local backup");
        }

        let mut local_replicas = utils::get_local_replicas(self.local_path.as_deref())
            .context("failed to get local replicas")?;

        utils::rotate_local_replicas(&mut local_replicas, self.replicas as usize)
            .context("failed to rotate local replicas")?;

        let next_replica_number = local_replicas
            .first()
            .map(|r| r.replica_number + 1)
            .unwrap_or(1);

        utils::create_local_backup(ctx, self.local_path.as_deref(), next_replica_number)
            .context("failed to create local backup")?;

        Ok(())
    }
}
