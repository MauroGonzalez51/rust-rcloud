use crate::{cli::parser::Args, config::prelude::*, log_debug, log_error, log_info, log_warn};
use anyhow::Context;

pub fn all_sync(
    args: &Args,
    registry: &mut Registry,
    tags: &[String],
    force_all: &bool,
) -> anyhow::Result<()> {
    log_debug!("using tags: {:?}", tags);

    let matching_paths_ids: Vec<String> = match tags.is_empty() {
        true => registry.paths.iter().map(|p| p.id.clone()).collect(),
        false => registry
            .paths
            .iter()
            .filter(|p| p.tags.iter().any(|t| tags.contains(t)))
            .map(|p| p.id.clone())
            .collect(),
    };

    log_info!("found {} path(s) to sync", matching_paths_ids.len());

    for path_id in matching_paths_ids {
        let path_info = registry
            .paths
            .iter()
            .find(|p| p.id == path_id)
            .map(|p| (p.local_path.clone(), p.remote_path.clone()));

        if let Some((local_path, remote_path)) = path_info {
            log_info!("Sync path: {} -> {}", local_path, remote_path);

            match crate::cli::commands::sync::handlers::path_sync::path_sync(
                args,
                registry,
                &None,
                &Some(path_id),
                force_all,
                &false,
            ) {
                Ok(_) => {
                    log_info!("synced {} -> {}", local_path, remote_path);
                }
                Err(err) => {
                    log_error!(
                        "an error ocurred while syncing {} -> {}: {}",
                        local_path,
                        remote_path,
                        err
                    );

                    let should_continue = inquire::Confirm::new("continue?")
                        .with_default(true)
                        .prompt()
                        .context("failed to get confirmation")?;

                    if !should_continue {
                        log_warn!("sync aborted by user");
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}
