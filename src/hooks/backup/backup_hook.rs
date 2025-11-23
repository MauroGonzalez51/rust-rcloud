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
        let local_path = self
            .local_path
            .as_deref()
            .context("remote path must be declared in order to perform a remote backup")?;

        let mut replicas =
            utils::get_local_replicas(local_path).context("failed to get local replicas")?;

        utils::rotate_local_replicas(&mut replicas, self.replicas as usize)
            .context("failed to rotate local replicas")?;

        let next_replica = replicas.first().map(|r| r.replica_number + 1).unwrap_or(1);

        utils::create_local_backup(ctx, local_path, next_replica)
            .context("failed to create local backup")?;

        Ok(())
    }

    pub fn backup_remote(&self, ctx: &HookContext) -> anyhow::Result<()> {
        let remote_path = self
            .remote_path
            .as_deref()
            .context("remote path must be declared in order to perform a remote backup")?;

        let mut replicas =
            utils::get_remote_replicas(remote_path, &ctx.rclone_path, &ctx.remote_config)
                .context("failed to get remote replicas")?;

        utils::rotate_remote_replicas(
            &mut replicas,
            self.replicas as usize,
            &ctx.rclone_path,
            &ctx.remote_config,
        )
        .context("failed to rotate remote replicas")?;

        let next_replica = replicas.first().map(|r| r.replica_number + 1).unwrap_or(1);

        utils::create_remote_backup(ctx, remote_path, next_replica)
            .context("failed to create remote backup")?;

        Ok(())
    }
}
