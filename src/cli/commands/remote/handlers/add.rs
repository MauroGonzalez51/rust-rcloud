use crate::{
    cli::{commands::remote::utils::remote, context::CommandContext},
    config::prelude::*,
    log_debug, log_success,
};
use anyhow::Context;
use uuid::Uuid;

pub struct LocalArgs<'a> {
    pub name: &'a Option<String>,
    pub provider: &'a Option<String>,
}

pub fn remote_add(mut context: CommandContext<LocalArgs>) -> anyhow::Result<()> {
    let remote_name = match context.local.name {
        Some(value) => value,
        None => &remote::Prompt::name()
            .with_help_message(
                "Must be the same that you inserted when configuring the remote in 'rcloud'",
            )
            .prompt()
            .context("failed to create prompt")?
            .clone(),
    };

    let provider = match context.local.provider {
        Some(value) => value,
        None => &remote::Prompt::provider()
            .prompt()
            .context("failed to create prompt")?
            .clone(),
    };

    log_debug!("[ INFO ] adding remote '{remote_name}' ({provider}) to registry");

    context
        .tx(|rgx| {
            rgx.remotes.push(Remote {
                id: Uuid::new_v4().to_string(),
                remote_name: remote_name.clone(),
                provider: provider.clone(),
            })
        })
        .context("error inside transaction")?;

    log_success!("remote added succesfully");

    Ok(())
}
