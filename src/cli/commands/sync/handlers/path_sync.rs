use crate::{
    cli::{commands::path::utils::path, parser::Args},
    config::prelude::{Hook, HookConfig, HookContext, HookExecType, Registry},
    log_debug, log_info, log_success,
};
use anyhow::Context;
use std::path::PathBuf;

fn compute_remote_filename(hooks: &[HookConfig], base_name: &str) -> String {
    if hooks.is_empty() {
        return base_name.to_string();
    }

    let last_hook = &hooks[hooks.len() - 1];

    match last_hook {
        HookConfig::Zip(_) => format!("{}.zip", base_name),
    }
}

pub fn path_sync(
    args: &Args,
    registry: &Registry,
    direction: &Option<HookExecType>,
    path_id: &Option<String>,
) -> anyhow::Result<()> {
    let direction = match direction {
        Some(value) => value,
        None => &HookExecType::select("Select direction:")
            .with_vim_mode(true)
            .prompt()
            .context("failed to select direction")?,
    };

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
            log_info!("running pre-transaction hooks");

            let mut context = HookContext::new(PathBuf::from(&path_config.local_path));

            for hook in hooks.push.iter() {
                let hook: Box<dyn Hook> = Box::from(hook.clone());
                context = hook.process(context)?;
            }

            let final_name = compute_remote_filename(
                &hooks.push,
                std::path::Path::new(&path_config.remote_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("archive"),
            );

            if args.debug {
                log_debug!("final_name: {:?}", final_name);
            }

            let final_path = match hooks.push.is_empty() {
                true => context.path.clone(),
                false => {
                    let renamed_path = context
                        .path
                        .parent()
                        .context("failed to get parent path")?
                        .join(&final_name);

                    if context.path.is_file() {
                        fs_extra::file::move_file(
                            &context.path,
                            &renamed_path,
                            &fs_extra::file::CopyOptions::new().overwrite(true),
                        )
                        .context("failed to move file")?;
                    }

                    if context.path.is_dir() {
                        fs_extra::dir::move_dir(
                            &context.path,
                            &renamed_path,
                            &fs_extra::dir::CopyOptions::new().overwrite(true),
                        )
                        .context("failed to move directory")?;
                    }

                    renamed_path
                }
            };

            if args.debug {
                log_debug!("final_path: {:?}", final_path);
            }

            let status = std::process::Command::new("rclone")
                .args([
                    "copy",
                    final_path
                        .to_str()
                        .context("failed to convert final-path to str")?,
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

            log_success!(
                "sent to remote {} -> {}:{}",
                path_config.local_path,
                remote_config.remote_name,
                path_config.remote_path
            );
        }

        HookExecType::Pull => {
            let temp_dir = tempfile::tempdir().context("failed to create temp directory")?;

            let remote_filename = compute_remote_filename(
                &hooks.push,
                std::path::Path::new(&path_config.remote_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("archive"),
            );

            if args.debug {
                log_debug!("remote_filename: {:?}", remote_filename);
            }

            let remote_file_path = match hooks.pull.is_empty() {
                true => format!("{}:{}", remote_config.remote_name, path_config.remote_path),
                false => format!(
                    "{}:{}/{}",
                    remote_config.remote_name, path_config.remote_path, remote_filename
                ),
            };

            if args.debug {
                log_debug!("remote_file_path: {:?}", remote_file_path)
            }

            let status = std::process::Command::new("rclone")
                .args([
                    "copy",
                    &remote_file_path,
                    temp_dir
                        .path()
                        .to_str()
                        .context("failed to convert tempdir path to str")?,
                    "--progress",
                    "--checksum",
                    "--transfers=4",
                    "--checkers=8",
                ])
                .status()
                .context("failed to execute rclone pull copy")?;

            if !status.success() {
                anyhow::bail!("rclone pull copy failed");
            }

            log_info!("running post-transaction hooks");

            let downloaded_file = match hooks.pull.is_empty() {
                true => temp_dir.path().to_path_buf(),
                false => {
                    let base_name = std::path::Path::new(&path_config.remote_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("archive");
                    temp_dir.path().join(format!("{}.zip", base_name))
                }
            };

            let mut context = HookContext::new(downloaded_file);
            for hook in hooks.pull.iter().rev() {
                let hook: Box<dyn Hook> = Box::from(hook.clone());
                context = hook.process(context)?;
            }

            log_info!("moving processed content to local_path");

            if args.debug {
                log_debug!(
                    "context path: {:?} (exists: {})",
                    context.path,
                    context.path.exists()
                );
            }

            if let Some(parent) = std::path::Path::new(&path_config.local_path).parent() {
                std::fs::create_dir_all(parent).context("failed to create parent directory")?;
            }

            if context.path.is_file() {
                fs_extra::file::move_file(
                    &context.path,
                    &path_config.local_path,
                    &fs_extra::file::CopyOptions::new(),
                )
                .with_context(|| format!("failed to move file to {}", path_config.local_path))?;
            }

            if context.path.is_dir() {
                if !std::path::Path::new(&path_config.local_path).exists() {
                    std::fs::create_dir_all(&path_config.local_path)
                        .context("failed to create destination directory")?;
                }

                fs_extra::dir::copy(
                    &context.path,
                    &path_config.local_path,
                    &fs_extra::dir::CopyOptions::new()
                        .overwrite(true)
                        .content_only(true),
                )
                .with_context(|| {
                    format!("failed to move directory to {}", path_config.local_path)
                })?;

                std::fs::remove_dir_all(&context.path)
                    .context("failed to remove temp directory")?;
            }

            log_success!(
                "pulled from remote {}:{} -> {}",
                remote_config.remote_name,
                path_config.remote_path,
                path_config.local_path
            );
        }
    }

    Ok(())
}
