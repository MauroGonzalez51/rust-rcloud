use crate::{hooks::prelude::HookContext, log_debug};

pub fn create_remote_backup(
    ctx: &HookContext,
    remote_path: &str,
    replica_number: u32,
) -> anyhow::Result<()> {
    let timestamp = chrono::Utc::now().timestamp();

    log_debug!("creating remote backup");

    let src = format!(
        "{}:{}",
        ctx.remote_config.remote_name, ctx.path_config.remote_path
    );
    let dst = format!(
        "{}:{}/{}.{}",
        ctx.remote_config.remote_name, remote_path, timestamp, replica_number
    );

    if !ctx.dependencies.rclone.copy_to(&src, &dst)? {
        log_debug!("remote source not found, skipping remote backup");
    }

    Ok(())
}
