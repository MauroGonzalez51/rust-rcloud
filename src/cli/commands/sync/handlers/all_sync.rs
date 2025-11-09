use anyhow::Context;

use crate::{cli::parser::Args, config::prelude::*, log_info};

pub fn all_sync(args: &Args, registry: &Registry, tags: &[String]) -> anyhow::Result<()> {
    log_info!("{:?}", tags);

    let matching_paths = match tags.is_empty() {
        true => registry.paths.iter().collect::<Vec<_>>(),
        false => registry
            .paths
            .iter()
            .filter(|p| p.tags.iter().any(|t| tags.contains(t)))
            .collect::<Vec<_>>(),
    };

    for path in matching_paths {
        log_info!("Sync path: {} -> {}", path.local_path, path.remote_path);

        crate::cli::commands::sync::handlers::path_sync::path_sync(
            args,
            registry,
            &None,
            &Some(path.id.clone()),
        )
        .with_context(|| {
            format!(
                "failed to sync path: {} -> {}",
                path.local_path, path.remote_path
            )
        })?;
    }

    Ok(())
}
