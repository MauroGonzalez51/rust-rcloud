use crate::{
    cli::{commands::path::utils::path, parser::Args},
    config::prelude::Registry,
};
use anyhow::Context;

fn execute_rclone(rclone_path: &str, path: &str) -> anyhow::Result<std::process::ExitStatus> {
    std::process::Command::new(rclone_path)
        .args(["lsf", path])
        .status()
        .with_context(|| format!("failed to execute rclone ls {}", path))
}

pub fn remote_ls(
    args: &Args,
    registry: &Registry,
    path: &Option<String>,
    path_config: &Option<String>,
) -> anyhow::Result<()> {
    if let Some(path) = path {
        let status = execute_rclone(&args.rclone, path)?;

        if !status.success() {
            anyhow::bail!("rclone remote ls failed");
        }

        return Ok(());
    };

    let path_id = match path_config {
        Some(id) => id,
        None => &path::Prompt::path_config("Select the path:", registry)
            .context("failed to select path")?,
    };

    let path_config = registry
        .paths
        .iter()
        .find(|p| p.id == *path_id)
        .ok_or_else(|| anyhow::anyhow!("path does not exists"))?;

    let remote_config = registry
        .remotes
        .iter()
        .find(|r| r.id == *path_config.remote_id)
        .ok_or_else(|| anyhow::anyhow!("remote does not exists"))?;

    let status = execute_rclone(
        &args.rclone,
        &format!("{}:{}", remote_config.remote_name, path_config.remote_path),
    )?;

    if !status.success() {
        anyhow::bail!("rclone remote ls failed");
    }

    Ok(())
}
