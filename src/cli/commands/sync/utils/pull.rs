use crate::{
    cli::commands::sync::utils,
    config::{
        prelude::{AppConfig, HookConfig, HookExecType, PathConfig, Registry},
        remote::Remote,
    },
    hooks::prelude::{HookContext, HookContextMetadata},
    log_debug, log_info, log_success, log_warn,
    utils::hash,
};
use anyhow::Context;

/// Path-related inputs for a pull.
pub struct PullOptionsPaths<'a> {
    /// Path to the `rclone` executable.
    pub rclone: &'a str,
    /// Source remote.
    pub remote: &'a Remote,
    /// Path configuration being pulled.
    pub path_config: &'a PathConfig,
}

/// Inputs for [`pull`].
pub struct PullOptions<'a> {
    /// Application configuration.
    pub config: &'a AppConfig,
    /// Shared registry (updated with the new hash on success).
    pub registry: std::sync::Arc<std::sync::Mutex<Registry>>,
    /// Path/remote/rclone inputs.
    pub paths: PullOptionsPaths<'a>,
    /// Pull hooks (applied in reverse of the push order).
    pub hooks: &'a [HookConfig],
    /// Skip the unchanged-content check when `true`.
    pub force: &'a bool,
    /// Remove the local destination before writing when `true`.
    pub clean: &'a bool,
}

/// Pulls a configured path from its remote to local storage.
///
/// Steps:
/// 1. Download the remote content into a temp directory via `rclone`.
/// 2. Run the pull hooks in reverse order (undoing the push pipeline:
///    decrypt, extract, ...).
/// 3. Hash the result and, unless `force`, skip when it matches the stored
///    hash. If `clean` is set, clear the destination first.
/// 4. Move the processed content to the local path and persist the new hash.
///
/// # Errors
/// Returns an error if the download, a hook, or the local move fails.
pub fn pull(options: PullOptions) -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir().context("failed to create temp directory")?;

    // Prefer the filename recorded by the push that produced the remote copy;
    // this is the single source of truth and avoids recomputing it from the
    // (possibly reordered) pull hook list. Fall back to computing it for paths
    // pushed before this was tracked.
    let remote_filename = utils::resolve_remote_filename(
        options.paths.path_config.remote_filename.as_deref(),
        options.hooks,
        &options.paths.path_config.remote_path,
    );

    let remote_path = match &remote_filename {
        None => format!(
            "{}:{}",
            options.paths.remote.remote_name, options.paths.path_config.remote_path
        ),
        Some(filename) => {
            let parent = std::path::Path::new(&options.paths.path_config.remote_path)
                .parent()
                .unwrap_or(std::path::Path::new(""));

            format!(
                "{}:{}",
                options.paths.remote.remote_name,
                parent.join(filename).to_string_lossy()
            )
        }
    };

    log_debug!("remote_path: {:?}", remote_path);

    let dependencies =
        crate::hooks::prelude::HookDependencies::production(options.paths.rclone);

    let status = dependencies.rclone.transfer(
        &remote_path,
        temp_dir
            .path()
            .to_str()
            .context("failed to convert tempdir path to str")?,
    )?;

    if !status.success() {
        anyhow::bail!("rclone pull copy failed");
    }

    log_info!("running post-transaction hooks");

    let downloaded_file = match &remote_filename {
        None => {
            let entries: Vec<_> = std::fs::read_dir(temp_dir.path())
                .context("failed to read temp directory")?
                .filter_map(Result::ok)
                .collect();

            match entries.len() {
                1 => entries[0].path(),
                _ => temp_dir.path().to_path_buf(),
            }
        }
        Some(filename) => {
            let candidate = temp_dir.path().join(filename);
            if candidate.exists() {
                candidate
            } else {
                let entries: Vec<_> = std::fs::read_dir(temp_dir.path())
                    .context("failed to read temp directory")?
                    .filter_map(Result::ok)
                    .collect();

                match entries.len() {
                    1 => entries[0].path(),
                    _ => temp_dir.path().to_path_buf(),
                }
            }
        }
    };

    log_debug!(
        "downloaded_file: {:?} (exists: {})",
        downloaded_file,
        downloaded_file.exists()
    );

    utils::check_hooks(options.hooks, &HookExecType::Pull);

    let reversed_hooks: Vec<_> = options.hooks.iter().rev().cloned().collect();
    let context = utils::execute_hooks(
        HookContext::new(
            downloaded_file,
            dependencies.clone(),
            options.paths.remote,
            options.paths.path_config,
        )
        .with_metadata(
            HookContextMetadata::SourceLocalPath,
            &options.paths.path_config.local_path,
        )
        .with_metadata(
            HookContextMetadata::SourceRemotePath,
            &options.paths.path_config.remote_path,
        ),
        &reversed_hooks,
        options.config,
    )?;

    let processed_hash = hash::Hash::hash_path(&context.path)
        .context("failed to calculate processed content hash")?;

    log_debug!("processed hash: {}", processed_hash);

    match utils::force(
        &HookExecType::Pull,
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
            log_info!("local path does not exist, proceding with sync");
        }
    }

    log_info!("moving processed content to local_path");

    utils::clean(
        &HookExecType::Pull,
        options.clean,
        &options.paths.path_config.local_path,
    )?;

    log_debug!(
        "context path: {:?} (exists: {})",
        context.path,
        context.path.exists()
    );

    if let Some(parent) = std::path::Path::new(&options.paths.path_config.local_path).parent() {
        std::fs::create_dir_all(parent).context("failed to create parent directory")?;
    }

    if context.path.is_file() {
        fs_extra::file::move_file(
            &context.path,
            &options.paths.path_config.local_path,
            &fs_extra::file::CopyOptions::new(),
        )
        .with_context(|| {
            format!(
                "failed to move file to {}",
                options.paths.path_config.local_path
            )
        })?;
    }

    if context.path.is_dir() {
        if !std::path::Path::new(&options.paths.path_config.local_path).exists() {
            std::fs::create_dir_all(&options.paths.path_config.local_path)
                .context("failed to create destination directory")?;
        }

        fs_extra::dir::copy(
            &context.path,
            &options.paths.path_config.local_path,
            &fs_extra::dir::CopyOptions::new()
                .overwrite(true)
                .content_only(true),
        )
        .with_context(|| {
            format!(
                "failed to move directory to {}",
                options.paths.path_config.local_path
            )
        })?;

        std::fs::remove_dir_all(&context.path).context("failed to remove temp directory")?;
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
            }
        })
        .context("failed to execute transaction")?;

    log_success!(
        "pulled from remote {}:{} -> {}",
        options.paths.remote.remote_name,
        options.paths.path_config.remote_path,
        options.paths.path_config.local_path
    );

    Ok(())
}
