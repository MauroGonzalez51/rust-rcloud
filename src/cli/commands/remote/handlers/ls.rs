use crate::cli::{commands::path::utils::path, context::CommandContext};
use anyhow::Context;

fn execute_rclone(rclone_path: &str, path: &str) -> anyhow::Result<std::process::ExitStatus> {
    std::process::Command::new(rclone_path)
        .args(["lsf", path])
        .status()
        .with_context(|| format!("failed to execute rclone ls {}", path))
}

#[derive(Clone)]
pub struct LocalArgs<'a> {
    pub path: &'a Option<String>,
    pub path_config: &'a Option<String>,
}

pub fn remote_ls(context: CommandContext<LocalArgs>) -> anyhow::Result<()> {
    if let Some(path) = context.local.path {
        let status = execute_rclone(&context.global.rclone, path)?;

        if !status.success() {
            anyhow::bail!("rclone remote ls failed");
        }

        return Ok(());
    };

    let path_id = match context.local.path_config {
        Some(id) => id,
        None => &path::Prompt::path_config("Select the path:", &context.registry)
            .context("failed to select path")?,
    };

    let path_config = context
        .registry
        .paths
        .iter()
        .find(|p| p.id == *path_id)
        .ok_or_else(|| anyhow::anyhow!("path does not exists"))?;

    let remote_config = context
        .registry
        .remotes
        .iter()
        .find(|r| r.id == *path_config.remote_id)
        .ok_or_else(|| anyhow::anyhow!("remote does not exists"))?;

    let status = execute_rclone(
        &context.global.rclone,
        &format!("{}:{}", remote_config.remote_name, path_config.remote_path),
    )?;

    if !status.success() {
        anyhow::bail!("rclone remote ls failed");
    }

    Ok(())
}
