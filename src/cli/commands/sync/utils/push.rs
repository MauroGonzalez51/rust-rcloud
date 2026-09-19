use crate::{
    cli::commands::sync::utils,
    config::prelude::{AppConfig, HookConfig, HookExecType, PathConfig, Registry, Remote},
    hooks::prelude::{HookContext, HookContextMetadata},
    log_debug, log_info, log_success, log_warn,
    utils::hash,
};
use anyhow::Context;
use std::path::PathBuf;

/// Path-related inputs for a push.
pub struct PushOptionsPaths<'a> {
    /// Path to the `rclone` executable.
    pub rclone: &'a str,
    /// Target remote.
    pub remote: &'a Remote,
    /// Path configuration being pushed.
    pub path_config: &'a PathConfig,
}

/// Inputs for [`push`].
pub struct PushOptions<'a> {
    /// Application configuration.
    pub config: &'a AppConfig,
    /// Shared registry (updated with the new hash on success).
    pub registry: std::sync::Arc<std::sync::Mutex<Registry>>,
    /// Path/remote/rclone inputs.
    pub paths: PushOptionsPaths<'a>,
    /// Push hooks, applied in order.
    pub hooks: &'a [HookConfig],
    /// Skip the unchanged-content check when `true`.
    pub force: &'a bool,
}

/// Pushes a configured path to its remote.
///
/// Steps:
/// 1. Hash the local content and, unless `force`, skip when it matches the
///    stored hash (nothing changed).
/// 2. Run the push hooks in order over a [`HookContext`], transforming the
///    content (compress, encrypt, back up, ...).
/// 3. Rename the processed output to the hook-adjusted remote filename
///    (see [`compute_remote_filename`](super::compute_remote_filename())).
/// 4. Upload via `rclone` and, on success, persist the new hash in the registry.
///
/// # Errors
/// Returns an error if hashing, a hook, the rename, or the rclone upload fails.
pub fn push(options: PushOptions) -> anyhow::Result<()> {
    log_info!("running pre-transaction hooks");

    let processed_hash = hash::Hash::hash_path(&std::path::PathBuf::from(
        &options.paths.path_config.local_path,
    ))
    .context("failed to calculate content hash")?;

    log_debug!("calculated hash: {}", processed_hash);

    match utils::force(
        &HookExecType::Push,
        options.force,
        options.paths.path_config,
        &processed_hash,
    ) {
        utils::ForceResult::Proceed => {}
        utils::ForceResult::HashMatch => {
            log_warn!("content unchanged (hash match). skipping");
            return Ok(());
        }
        utils::ForceResult::PathNotFound => {
            unreachable!();
        }
    }

    utils::check_hooks(options.hooks, &HookExecType::Push);

    let dependencies =
        crate::hooks::prelude::HookDependencies::production(options.paths.rclone);

    let context = utils::execute_hooks(
        HookContext::new(
            PathBuf::from(&options.paths.path_config.local_path),
            dependencies.clone(),
            options.paths.remote,
            options.paths.path_config,
        )
        .with_metadata(HookContextMetadata::CalculatedHash, &processed_hash),
        options.hooks,
        options.config,
    )?;

    let final_name = utils::compute_remote_filename(
        options.hooks,
        std::path::Path::new(&options.paths.path_config.remote_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("archive"),
    );

    if !context.path.exists() {
        anyhow::bail!("processed file does not exists: {:?}", context.path);
    }

    log_debug!("final_name: {:?}", final_name);

    let final_path = match options.paths.path_config.local_path
        == context
            .path
            .to_str()
            .context("failed to convert context.path to str")?
    {
        true => {
            log_debug!("path unchanged, using original");
            PathBuf::from(&options.paths.path_config.local_path)
        }
        false => {
            log_debug!("path changed by hooks, renaming to final_name");

            let renamed_path = context
                .path
                .parent()
                .with_context(|| format!("failed to get parent path for: {:?}", context.path))?
                .join(&final_name);

            if renamed_path.exists() {
                log_debug!("removing existing target path from previous runs");
                match renamed_path.is_dir() {
                    true => std::fs::remove_dir_all(&renamed_path)
                        .context("failed to clean up existing temp directory")?,
                    false => std::fs::remove_file(&renamed_path)
                        .context("failed to clean up existing temp file")?,
                }
            }

            if context.path.is_file() && std::fs::rename(&context.path, &renamed_path).is_err() {
                fs_extra::file::move_file(
                    &context.path,
                    &renamed_path,
                    &fs_extra::file::CopyOptions::new().overwrite(true),
                )
                .context("failed to move file")?;
            }

            if context.path.is_dir() && std::fs::rename(&context.path, &renamed_path).is_err() {
                fs_extra::dir::move_dir(
                    &context.path,
                    &renamed_path,
                    &fs_extra::dir::CopyOptions::new().overwrite(true),
                )
                .with_context(|| {
                    format!(
                        "failed to move directory {} to {}",
                        context.path.display(),
                        renamed_path.display()
                    )
                })?;
            }

            renamed_path
        }
    };

    log_debug!("final_path: {:?}", final_path);

    let remote_dest = if final_name != options.paths.path_config.remote_path {
        let parent = std::path::Path::new(&options.paths.path_config.remote_path)
            .parent()
            .unwrap_or(std::path::Path::new(""));

        format!(
            "{}:{}",
            options.paths.remote.remote_name,
            parent.join(&final_name).to_string_lossy()
        )
    } else {
        format!(
            "{}:{}",
            options.paths.remote.remote_name, options.paths.path_config.remote_path
        )
    };

    let status = dependencies.rclone.transfer(
        final_path
            .to_str()
            .context("failed to convert final_path to str")?,
        &remote_dest,
    )?;

    if !status.success() {
        anyhow::bail!("rclone push sync failed");
    }

    options
        .registry
        .lock()
        .map_err(|e| anyhow::anyhow!("{}", e))?
        .tx(|rgx| {
            if let Some(path) = rgx
                .paths
                .iter_mut()
                .find(|p| p.id == options.paths.path_config.id)
            {
                path.hash = Some(processed_hash);
                path.remote_filename = Some(final_name.clone());
            }
        })
        .context("failed to execute transaction")?;

    log_success!(
        "sent to remote {} -> {}",
        options.paths.path_config.local_path,
        remote_dest
    );

    Ok(())
}
