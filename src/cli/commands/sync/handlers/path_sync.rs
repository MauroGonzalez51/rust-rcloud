use crate::{
    cli::{commands::path::utils::path, parser::Args},
    config::prelude::{HookExecType, Registry},
};
use anyhow::Context;

pub fn path_sync(
    _args: &Args,
    registry: &Registry,
    direction: &HookExecType,
    path_id: &Option<String>,
) -> anyhow::Result<()> {
    let path_id = match path_id {
        Some(value) => value.clone(),
        None => path::Prompt::path_config("Select the path to sync:", registry)
            .context("failed to select remote")?
            .clone(),
    };

    let path_config = registry
        .paths
        .iter()
        .find(|p| p.id == path_id)
        .ok_or_else(|| anyhow::anyhow!("path does not exists"))?;

    let remote_config = registry
        .remotes
        .iter()
        .find(|r| r.id == path_config.remote_id)
        .ok_or_else(|| anyhow::anyhow!("remote does not exists"))?;

    let hooks = &path_config.hooks;

    match direction {
        HookExecType::Push => {
            if hooks.push.is_empty() {
                let status = std::process::Command::new("rclone")
                    .args([
                        "sync",
                        &path_config.local_path,
                        &format!("{}:{}", remote_config.remote_name, path_config.remote_path),
                        "--progress",
                        "--checksum",
                        "--delete-during",
                        "--transfers=8",
                        "--checkers=16",
                    ])
                    .status()
                    .context("failed to execute rclone push sync")?;

                if !status.success() {
                    anyhow::bail!("rclone push sync failed");
                }
            }
        }
        HookExecType::Pull => {
            if hooks.pull.is_empty() {
                let status = std::process::Command::new("rclone")
                    .args([
                        "copy",
                        &format!("{}:{}", remote_config.remote_name, path_config.remote_path),
                        &path_config.local_path,
                        "--progress",
                        "--checksum",
                        "--transfers=4",
                        "--checkers=8",
                        "--update",
                    ])
                    .status()
                    .context("failed to execute rclone pull copy")?;

                if !status.success() {
                    anyhow::bail!("rclone pull copy failed");
                }
            }
        }
    }

    Ok(())
}
