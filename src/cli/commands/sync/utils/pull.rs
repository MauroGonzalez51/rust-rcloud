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

pub struct PullOptionsPaths<'a> {
    pub rclone: &'a str,
    pub remote: &'a Remote,
    pub path_config: &'a PathConfig,
}

pub struct PullOptions<'a> {
    pub config: &'a AppConfig,
    pub registry: &'a mut Registry,
    pub paths: PullOptionsPaths<'a>,
    pub hooks: &'a [HookConfig],
    pub force: &'a bool,
    pub clean: &'a bool,
}

pub fn pull(options: PullOptions) -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir().context("failed to create temp directory")?;

    let remote_filename = match options.hooks.iter().any(|h| h.modifies_filename()) {
        true => Some(utils::compute_remote_filename(
            options.hooks,
            std::path::Path::new(&options.paths.path_config.remote_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("archive"),
        )),
        false => None,
    };

    let remote_path = match &remote_filename {
        None => format!(
            "{}:{}",
            options.paths.remote.remote_name, options.paths.path_config.remote_path
        ),
        Some(filename) => {
            format!(
                "{}:{}/{}",
                options.paths.remote.remote_name, options.paths.path_config.remote_path, filename
            )
        }
    };

    log_debug!("remote_path: {:?}", remote_path);

    let status = utils::execute_rclone(
        options.paths.rclone,
        &remote_path,
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
        Some(filename) => temp_dir.path().join(filename),
    };

    log_debug!(
        "downloaded_file: {:?} (exists: {})",
        downloaded_file,
        downloaded_file.exists()
    );

    let reversed_hooks: Vec<HookConfig> = options.hooks.iter().rev().cloned().collect();
    let context = utils::execute_hooks(
        HookContext::new(
            downloaded_file,
            options.paths.rclone,
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
