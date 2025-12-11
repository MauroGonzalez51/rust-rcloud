use crate::{
    cli::{
        commands::path::utils::{hooks, path, tags},
        context::CommandContext,
    },
    config::prelude::*,
    log_debug, log_warn, utils,
};
use anyhow::Context;
use inquire::Confirm;

#[derive(Clone)]
pub struct LocalArgs<'a> {
    pub remote_id: &'a Option<String>,
    pub local_path: &'a Option<String>,
    pub remote_path: &'a Option<String>,
}

pub fn path_add(mut context: CommandContext<LocalArgs>) -> anyhow::Result<()> {
    if context.registry.remotes.is_empty() {
        log_warn!("there are no remotes configured");
        return Ok(());
    }

    let remote_id = match context.local.remote_id {
        Some(value) => value,
        None => &path::Prompt::remote_id::<
            fn(inquire::Select<'_, String>) -> inquire::Select<'_, String>,
        >(&context.registry, None)
        .context("failed to get remote_id")?,
    };

    let local_path = match context.local.local_path {
        Some(value) => value,
        None => &path::Prompt::path("local path:")
            .prompt()
            .context("failed to get local path")?,
    };

    let local_path = utils::path::expand_path(local_path)?
        .to_string_lossy()
        .to_string();

    let remote_path = match context.local.remote_path {
        Some(value) => value,
        None => &path::Prompt::path("remote path:")
            .prompt()
            .context("failed to get remote path")?,
    };

    log_debug!(
        "{} -> (remote_id: {}):{}",
        local_path,
        remote_id,
        remote_path
    );

    let (push, pull) = hooks::declare_hooks().context("failed to get hooks")?;

    let tags = tags::declare_tags(&context.registry).context("failed to get tags")?;

    let path_config = PathConfig {
        id: uuid::Uuid::new_v4().to_string(),
        remote_id: remote_id.clone(),
        local_path: local_path.clone(),
        remote_path: remote_path.clone(),
        hash: None,
        hooks: PathConfigHooks { push, pull },
        tags,
    };

    log_debug!("using path_config: {:?}", path_config);

    let confirm_save = Confirm::new("Save this configuration?")
        .with_default(true)
        .prompt()
        .context("failed to get confirmation")?;

    if confirm_save {
        context
            .registry
            .tx(|rgx| {
                rgx.paths.push(path_config);
            })
            .context("failed to execute transaction")?;
    }

    Ok(())
}
