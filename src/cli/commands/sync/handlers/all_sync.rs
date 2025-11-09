use anyhow::Context;

use crate::{cli::parser::Args, config::prelude::*, log_info};

pub fn all_sync(
    args: &Args,
    registry: &mut Registry,
    tags: &[String],
    force_all: &bool,
) -> anyhow::Result<()> {
    log_info!("{:?}", tags);

    let matching_paths_ids: Vec<String> = match tags.is_empty() {
        true => registry.paths.iter().map(|p| p.id.clone()).collect(),
        false => registry
            .paths
            .iter()
            .filter(|p| p.tags.iter().any(|t| tags.contains(t)))
            .map(|p| p.id.clone())
            .collect(),
    };

    for path_id in matching_paths_ids {
        let path_info = registry
            .paths
            .iter()
            .find(|p| p.id == path_id)
            .map(|p| (p.local_path.clone(), p.remote_path.clone()));

        if let Some((local_path, remote_path)) = path_info {
            log_info!("Sync path: {} -> {}", local_path, remote_path);

            crate::cli::commands::sync::handlers::path_sync::path_sync(
                args,
                registry,
                &None,
                &Some(path_id),
                force_all,
            )
            .with_context(|| format!("failed to sync path: {} -> {}", local_path, remote_path))?;
        }
    }

    Ok(())
}
