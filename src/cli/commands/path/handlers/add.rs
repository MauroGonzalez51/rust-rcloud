use crate::{
    cli::{
        commands::path::utils::{hooks, path, tags},
        context::CommandContext,
        output::OutputSink,
        prompter::Prompter,
    },
    config::prelude::*,
    log_debug, log_warn, utils,
};
use anyhow::Context;

#[derive(Clone)]
pub struct LocalArgs<'a> {
    pub remote_id: &'a Option<String>,
    pub local_path: &'a Option<String>,
    pub remote_path: &'a Option<String>,
}

impl<'a> Default for LocalArgs<'a> {
    fn default() -> Self {
        Self {
            remote_id: &None,
            local_path: &None,
            remote_path: &None,
        }
    }
}

pub fn path_add<P: Prompter, S: OutputSink>(
    context: CommandContext<LocalArgs>,
    prompter: &P,
    sink: &S,
) -> anyhow::Result<()> {
    if context.with_registry()?.remotes.is_empty() {
        log_warn!("there are no remotes configured");
        sink.warn("there are no remotes configured");
        return Ok(());
    }

    let remote_id = match context.local.remote_id {
        Some(value) => value.clone(),
        None => path::Prompt::remote_id(prompter, std::sync::Arc::clone(&context.registry))
            .context("failed to get remote_id")?,
    };

    let local_path = match context.local.local_path {
        Some(value) => value.clone(),
        None => path::Prompt::path(prompter, "local path:")
            .context("failed to get local path")?,
    };

    let local_path = utils::expand_path(&local_path)?
        .to_string_lossy()
        .to_string();

    let remote_path = match context.local.remote_path {
        Some(value) => value.clone(),
        None => path::Prompt::path(prompter, "remote path:")
            .context("failed to get remote path")?,
    };

    log_debug!(
        "{} -> (remote_id: {}):{}",
        local_path,
        remote_id,
        remote_path
    );

    let (push, pull) = hooks::declare_hooks(prompter).context("failed to get hooks")?;

    let tags = tags::declare_tags(prompter, std::sync::Arc::clone(&context.registry))
        .context("failed to get tags")?;

    let path_config = PathConfig {
        id: uuid::Uuid::new_v4().to_string(),
        remote_id: remote_id.clone(),
        local_path: local_path.clone(),
        remote_path: remote_path.clone(),
        hash: None,
        remote_filename: None,
        hooks: PathConfigHooks { push, pull },
        tags,
    };

    log_debug!("using path_config: {:?}", path_config);

    let confirm_save = prompter
        .confirm("Save this configuration?", true)
        .context("failed to get confirmation")?;

    if confirm_save {
        context
            .with_registry()?
            .tx(|rgx| rgx.paths.push(path_config))
            .context("failed to execute transaction")?;
        sink.success("path added successfully");
    } else {
        sink.info("path not saved");
    }

    Ok(())
}
