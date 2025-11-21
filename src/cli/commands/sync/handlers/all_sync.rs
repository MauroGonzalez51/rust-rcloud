use crate::{
    cli::{commands::sync::handlers::path_sync, context::CommandContext},
    log_debug, log_error, log_info, log_warn,
};
use anyhow::Context;

pub struct LocalArgs<'a> {
    pub tags: &'a [String],
    pub force_all: &'a bool,
    pub clean_all: &'a bool,
}

pub fn all_sync(mut context: CommandContext<LocalArgs>) -> anyhow::Result<()> {
    log_debug!("using tags: {:?}", context.local.tags);

    let matching_paths_ids: Vec<String> = match context.local.tags.is_empty() {
        true => context.paths.iter().map(|p| p.id.clone()).collect(),
        false => context
            .paths
            .iter()
            .filter(|p| p.tags.iter().any(|t| context.local.tags.contains(t)))
            .map(|p| p.id.clone())
            .collect(),
    };

    log_info!("found {} path(s) to sync", matching_paths_ids.len());

    for path_id in matching_paths_ids {
        let path_info = context
            .paths
            .iter()
            .find(|p| p.id == path_id)
            .map(|p| (p.local_path.clone(), p.remote_path.clone()));

        if let Some((local_path, remote_path)) = path_info {
            log_info!("Sync path: {} -> {}", local_path, remote_path);

            let args = path_sync::LocalArgs {
                direction: &None,
                path_id: &Some(path_id),
                force: context.local.force_all,
                clean: context.local.clean_all,
            };

            let path_context = CommandContext::new(
                context.global.clone(),
                std::mem::take(&mut context.registry),
                args,
            );

            match crate::cli::commands::sync::handlers::path_sync::path_sync(path_context) {
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
