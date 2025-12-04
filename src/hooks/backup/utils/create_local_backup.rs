use crate::{hooks::prelude::HookContext, log_debug};
use anyhow::Context;

pub fn create_local_backup(
    ctx: &HookContext,
    local_path: &str,
    replica_number: u32,
) -> anyhow::Result<()> {
    let directory = std::path::Path::new(local_path);

    if !directory.exists() {
        std::fs::create_dir_all(directory)
            .with_context(|| format!("failed to create backup directory: {:?}", directory))?;
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .context("failed to get timestap")?
        .as_secs();

    let backup_path = directory.join(format!("{}.{}", timestamp, replica_number));

    log_debug!("writing files to backup path: {:?}", backup_path);

    if ctx.path.is_file() {
        std::fs::copy(&ctx.path, &backup_path)
            .with_context(|| format!("failed to copy file to backup: {:?}", backup_path))?;

        return Ok(());
    }

    if ctx.path.is_dir() {
        std::fs::create_dir_all(&backup_path)
            .with_context(|| format!("failed to create backup directory: {:?}", backup_path))?;

        fs_extra::dir::copy(&ctx.path, &backup_path, &fs_extra::dir::CopyOptions::new())
            .with_context(|| format!("failed to copy directory to backup: {:?}", backup_path))?;

        return Ok(());
    }

    Ok(())
}
