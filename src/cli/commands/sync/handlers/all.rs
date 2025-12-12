use crate::{
    cli::{
        commands::{path::utils::tags, sync::handlers::single},
        context::CommandContext,
    },
    log_error, log_info, log_warn,
};
use anyhow::Context;

#[derive(Clone, Default)]
pub struct LocalArgs<'a> {
    pub tags: &'a [String],
}

pub fn sync_all(mut context: CommandContext<LocalArgs>) -> anyhow::Result<()> {
    let tags = tags::select_tags(std::sync::Arc::clone(&context.registry))?;

    let matching_paths_ids: Vec<String> = match tags.is_empty() {
        true => context
            .with_registry()?
            .paths
            .iter()
            .map(|p| p.id.clone())
            .collect(),
        false => context
            .with_registry()?
            .paths
            .iter()
            .filter(|p| p.tags.iter().any(|t| context.local.tags.contains(t)))
            .map(|p| p.id.clone())
            .collect(),
    };

    log_info!("found {} path(s) to sync", matching_paths_ids.len());

    for path_id in matching_paths_ids {
        let path_info = context
            .with_registry()?
            .paths
            .iter()
            .find(|p| p.id == path_id)
            .map(|p| (p.local_path.clone(), p.remote_path.clone()));

        if let Some((local_path, remote_path)) = path_info {
            log_info!("Sync path: {} -> {}", local_path, remote_path);

            let args = single::LocalArgs {
                direction: &None,
                path_id: &Some(path_id),
                force: &None,
                clean: &None,
            };

            let path_context = context.with_args(args);

            match single::sync_single(path_context) {
                Ok(_context) => {
                    context.registry = _context.registry;
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
