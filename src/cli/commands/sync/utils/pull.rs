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

pub fn pull(
    registry: &mut Registry,
    rclone_path: &str,
    remote_config: &Remote,
    path_config: &PathConfig,
    hooks: &[HookConfig],
    force: &bool,
) -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir().context("failed to create temp directory")?;

    let remote_filename = utils::compute_remote_filename::compute_remote_filename(
        hooks,
        std::path::Path::new(&path_config.remote_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("archive"),
    );

    log_debug!("remote_filename: {:?}", remote_filename);

    let remote_file_path = match hooks.is_empty() {
        true => format!("{}:{}", remote_config.remote_name, path_config.remote_path),
        false => format!(
            "{}:{}/{}",
            remote_config.remote_name, path_config.remote_path, remote_filename
        ),
    };

    log_debug!("remote_file_path: {:?}", remote_file_path);

    let status = utils::execute_rclone::execute_rclone(
        rclone_path,
        &remote_file_path,
        temp_dir
            .path()
            .to_str()
            .context("failed to convert tempdir path to str")?,
        None,
    )?;

    if !status.success() {
        anyhow::bail!("rclone pull copy failed");
    }

    log_info!("running post-transaction hooks");

    let downloaded_file = match hooks.is_empty() {
        true => temp_dir.path().to_path_buf(),
        false => {
            let remote_filename = utils::compute_remote_filename::compute_remote_filename(
                hooks,
                std::path::Path::new(&path_config.remote_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("archive"),
            );

            temp_dir.path().join(remote_filename)
        }
    };

    let reversed_hooks: Vec<HookConfig> = hooks.iter().rev().cloned().collect();
    let context = utils::execute_hooks::execute_hooks(
        HookContext::new(downloaded_file, rclone_path, remote_config),
        &reversed_hooks,
    )?;

    let processed_hash = hash::Hash::hash_path(&context.path)
        .context("failed to calculate processed content hash")?;

    log_debug!("processed hash: {}", processed_hash);

    match utils::options::force(&HookExecType::Pull, force, path_config, &processed_hash) {
        utils::options::ForceResult::Proceed => {}
        utils::options::ForceResult::HashMatch => {
            log_warn!("content unchanged (hash match). skipping");
            return Ok(());
        }
        utils::options::ForceResult::PathNotFound => {
            log_info!("local path does not exist, proceding with sync");
        }
    }

    log_info!("moving processed content to local_path");

    log_debug!(
        "context path: {:?} (exists: {})",
        context.path,
        context.path.exists()
    );

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
        .with_context(|| format!("failed to move directory to {}", path_config.local_path))?;

        std::fs::remove_dir_all(&context.path).context("failed to remove temp directory")?;
    }

    registry
        .tx(|rgx| {
            if let Some(path) = rgx.paths.iter_mut().find(|p| p.id == path_config.id) {
                path.hash = Some(processed_hash);
            }
        })
        .context("failed to execute transaction")?;

    log_success!(
        "pulled from remote {}:{} -> {}",
        remote_config.remote_name,
        path_config.remote_path,
        path_config.local_path
    );

    Ok(())
}
