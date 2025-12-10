use crate::{hooks::prelude::HookContext, log_debug};
use anyhow::Context;

pub fn create_remote_backup(
    ctx: &HookContext,
    remote_path: &str,
    replica_number: u32,
) -> anyhow::Result<()> {
    let timestamp = chrono::Utc::now().timestamp();

    log_debug!("creating remote backup");

    let output = std::process::Command::new(&ctx.rclone_path)
        .args([
            "copyto",
            &format!(
                "{}:{}",
                ctx.remote_config.remote_name, ctx.path_config.remote_path
            ),
            &format!(
                "{}:{}/{}",
                ctx.remote_config.remote_name,
                remote_path,
                &format!("{}.{}", timestamp, replica_number)
            ),
        ])
        .output()
        .context("failed to execute backup in remote")?;

    if !output.status.success() {
        log_debug!("remote source not found, skipping remote backup");
        return Ok(());
    }

    Ok(())
}
