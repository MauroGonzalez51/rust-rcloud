use crate::{
    cli::commands::sync::utils,
    config::{
        prelude::{HookConfig, HookContext, HookExecType, PathConfig, Registry},
        remote::Remote,
    },
    log_debug, log_info, log_success, log_warn,
    utils::hash,
};
use anyhow::Context;
use std::path::PathBuf;

pub fn push(
    registry: &mut Registry,
    rclone_path: &str,
    remote_config: &Remote,
    path_config: &PathConfig,
    hooks: &[HookConfig],
    force: &bool,
) -> anyhow::Result<()> {
    log_info!("running pre-transaction hooks");

    let processed_hash = hash::Hash::hash_path(&std::path::PathBuf::from(&path_config.local_path))
        .context("failed to calculate content hash")?;

    log_debug!("calculated hash: {}", processed_hash);

    match utils::options::force(&HookExecType::Push, force, path_config, &processed_hash) {
        utils::options::ForceResult::Proceed => {}
        utils::options::ForceResult::HashMatch => {
            log_warn!("content unchanged (hash match). skipping");
            return Ok(());
        }
        utils::options::ForceResult::PathNotFound => {
            unreachable!();
        }
    }

    let context = utils::execute_hooks::execute_hooks(
        HookContext::new(
            PathBuf::from(&path_config.local_path),
            rclone_path,
            remote_config,
        ),
        hooks,
    )?;

    let final_name = utils::compute_remote_filename::compute_remote_filename(
        hooks,
        std::path::Path::new(&path_config.remote_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("archive"),
    );

    log_debug!("final_name: {:?}", final_name);

    let final_path = match hooks.is_empty() {
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

    log_debug!("final_path: {:?}", final_path);

    let status = utils::execute_rclone::execute_rclone(
        rclone_path,
        final_path
            .to_str()
            .context("failed to convert final_path to str")?,
        &format!("{}:{}", remote_config.remote_name, path_config.remote_path),
        None,
    )?;

    if !status.success() {
        anyhow::bail!("rclone push sync failed");
    }

    registry
        .tx(|rgx| {
            if let Some(path) = rgx.paths.iter_mut().find(|p| p.id == path_config.id) {
                path.hash = Some(processed_hash);
            }
        })
        .context("failed to execute transaction")?;

    log_success!(
        "sent to remote {} -> {}:{}",
        path_config.local_path,
        remote_config.remote_name,
        path_config.remote_path
    );

    Ok(())
}
