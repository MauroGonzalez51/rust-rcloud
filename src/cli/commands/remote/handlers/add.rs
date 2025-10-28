use crate::{
    cli::{commands::remote::utils::remote, parser::Args},
    config::prelude::*,
    log_success,
};
use anyhow::Context;
use uuid::Uuid;

pub fn remote_add(
    args: &Args,
    registry: &mut Registry,
    name: &Option<String>,
    provider: &Option<String>,
) -> anyhow::Result<()> {
    let name = match name {
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

    if args.verbose > 0 {
        println!("[ INFO ] adding remote '{name}' ({provider}) to registry")
    }

    registry
        .tx(|rgx| {
            rgx.remotes.push(Remote {
                id: Uuid::new_v4().to_string(),
                remote_name: name.to_string(),
                provider: provider.to_string(),
            })
        })
        .context("error inside transaction")?;

    log_success!("remote added succesfully");

    Ok(())
}
