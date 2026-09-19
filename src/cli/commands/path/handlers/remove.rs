use crate::{
    cli::{commands::path::utils::path, context::CommandContext, prompter::Prompter},
    log_info, log_success, log_warn,
};
use anyhow::Context;

#[derive(Clone)]
pub struct LocalArgs<'a> {
    pub path_id: &'a Option<String>,
}

impl<'a> Default for LocalArgs<'a> {
    fn default() -> Self {
        Self { path_id: &None }
    }
}

pub fn path_remove<P: Prompter>(
    context: CommandContext<LocalArgs>,
    prompter: &P,
) -> anyhow::Result<()> {
    if context.with_registry()?.paths.is_empty() {
        log_warn!("no paths configured");
        return Ok(());
    }

    let path_id = match context.local.path_id {
        Some(value) => value.clone(),
        None => path::Prompt::path_config(
            prompter,
            "Select a record:",
            std::sync::Arc::clone(&context.registry),
        )
        .context("failed to select path config")?,
    };

    let path = context
        .with_registry()?
        .paths
        .iter()
        .find(|p| p.id == path_id)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("path not found"))?;

    log_info!("Removing path: {} -> {}", path.local_path, path.remote_path);

    context
        .with_registry()?
        .tx(|rgx| {
            rgx.paths.retain(|r| r.id != path.id);
        })
        .context("failed to execute transaction")?;

    log_success!("path removed successfully");

    Ok(())
}
