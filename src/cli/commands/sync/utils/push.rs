use crate::{
    cli::commands::sync::utils,
    config::prelude::{AppConfig, HookConfig, HookExecType, PathConfig, Registry, Remote},
    hooks::prelude::{HookContext, HookContextMetadata},
    log_debug, log_info, log_success, log_warn,
    utils::hash,
};
use anyhow::Context;
use std::path::PathBuf;

pub struct PushOptionsPaths<'a> {
    pub rclone: &'a str,
    pub remote: &'a Remote,
    pub path_config: &'a PathConfig,
}

pub struct PushOptions<'a> {
    pub config: &'a AppConfig,
    pub registry: &'a mut Registry,
    pub paths: PushOptionsPaths<'a>,
    pub hooks: &'a [HookConfig],
    pub force: &'a bool,
}

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

    let context = utils::execute_hooks(
        HookContext::new(
            PathBuf::from(&options.paths.path_config.local_path),
            options.paths.rclone,
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

    let final_path = match options.paths.path_config.local_path == context.path {
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

    let status = utils::execute_rclone(
        options.paths.rclone,
        final_path
            .to_str()
            .context("failed to convert final_path to str")?,
        &format!(
            "{}:{}",
            options.paths.remote.remote_name, options.paths.path_config.remote_path
        ),
        None,
    )?;

    if !status.success() {
        anyhow::bail!("rclone push sync failed");
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
        "sent to remote {} -> {}:{}",
        options.paths.path_config.local_path,
        options.paths.remote.remote_name,
        options.paths.path_config.remote_path
    );

    Ok(())
}
