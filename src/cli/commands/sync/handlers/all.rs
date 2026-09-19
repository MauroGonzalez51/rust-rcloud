use crate::{
    cli::{
        commands::{path::utils::tags, sync::handlers::single},
        context::CommandContext,
        output::OutputSink,
        prompter::Prompter,
    },
    log_error, log_info,
};
use anyhow::Context;

#[derive(Clone, Default)]
pub struct LocalArgs<'a> {
    pub tags: &'a [String],
}

pub fn sync_all<P: Prompter, S: OutputSink>(
    mut context: CommandContext<LocalArgs>,
    prompter: &P,
    sink: &S,
) -> anyhow::Result<()> {
    let tags = match context.local.tags.is_empty() {
        true => tags::select_tags(prompter, std::sync::Arc::clone(&context.registry))?,
        false => context.local.tags.to_vec(),
    };

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
            .filter(|p| p.tags.iter().any(|t| tags.contains(t)))
            .map(|p| p.id.clone())
            .collect(),
    };

    sink.info(format!("found {} path(s) to sync", matching_paths_ids.len()));

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

            match single::sync_single(path_context, prompter, sink) {
                Ok(_context) => {
                    context.registry = _context.registry;
                }
                Err(err) => {
                    log_error!(
                        "an error ocurred while syncing {} -> {}: {}",
                        local_path,
                        remote_path,
                        err
                    );
                    sink.error(format!(
                        "error syncing {} -> {}: {}",
                        local_path, remote_path, err
                    ));

                    let should_continue = prompter
                        .confirm("continue?", true)
                        .context("failed to get confirmation")?;

                    if !should_continue {
                        sink.warn("sync aborted by user");
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}
