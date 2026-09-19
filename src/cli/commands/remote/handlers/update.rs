use crate::{
    cli::{commands::remote::utils::remote, context::CommandContext, prompter::Prompter},
    log_debug, log_info, log_success, log_warn,
};
use anyhow::Context;

#[derive(Clone)]
pub struct LocalArgs<'a> {
    pub id: &'a Option<String>,
    pub name: &'a Option<String>,
    pub provider: &'a Option<String>,
}

impl<'a> Default for LocalArgs<'a> {
    fn default() -> Self {
        Self {
            id: &None,
            name: &None,
            provider: &None,
        }
    }
}

pub fn remote_update<P: Prompter>(
    context: CommandContext<LocalArgs>,
    prompter: &P,
) -> anyhow::Result<()> {
    if context.with_registry()?.remotes.is_empty() {
        log_warn!("no remotes configured");
        return Ok(());
    }

    let remote_info = match context.local.id {
        Some(value) => {
            if !context
                .with_registry()?
                .remotes
                .iter()
                .any(|r| r.id == *value)
            {
                anyhow::bail!("remote with id '{}' not found", value);
            }

            remote::Utils::remote_by_id(std::sync::Arc::clone(&context.registry), value)
                .context("remote not found")?
        }
        None => remote::Prompt::remote(
            prompter,
            "Select a remote to update:",
            std::sync::Arc::clone(&context.registry),
        )
        .context("failed to execute prompt")?,
    };

    log_debug!("using remote_info: {:?}", remote_info);

    let name = match context.local.name {
        Some(value) => value.clone(),
        None => remote::Prompt::name_with_default(prompter, &remote_info.remote_name)
            .context("failed to execute prompt")?,
    };

    let provider = match context.local.provider {
        Some(value) => value.clone(),
        None => remote::Prompt::provider_with_default(prompter, &remote_info.provider)
            .context("failed to create text prompt")?,
    };

    context
        .with_registry()?
        .tx(|rgx| {
            if let Some(remote) = rgx.remotes.iter_mut().find(|r| r.id == *remote_info.id) {
                log_info!("found remote to update");
                remote.remote_name = name.clone();
                remote.provider = provider.clone();
            }
        })
        .context("failed to execute transaction")?;

    log_success!("remote updated succesfully");

    Ok(())
}
