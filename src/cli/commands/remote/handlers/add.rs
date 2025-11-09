use crate::{
    cli::{commands::remote::utils::remote, parser::Args},
    config::prelude::*,
    log_debug, log_success,
};
use anyhow::Context;
use uuid::Uuid;

pub fn remote_add(
    _args: &Args,
    registry: &mut Registry,
    name: &Option<String>,
    provider: &Option<String>,
) -> anyhow::Result<()> {
    let remote_name = match name {
        Some(value) => value,
        None => &remote::Prompt::name()
            .with_help_message(
                "Must be the same that you inserted when configuring the remote in 'rcloud'",
            )
            .prompt()
            .context("failed to create prompt")?
            .clone(),
    };

    let provider = match provider {
        Some(value) => value,
        None => &remote::Prompt::provider()
            .prompt()
            .context("failed to create prompt")?
            .clone(),
    };

    log_debug!("[ INFO ] adding remote '{remote_name}' ({provider}) to registry");

    registry
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
