use crate::{
    cli::{commands::remote::utils::remote, parser::Args},
    config::prelude::*,
    log_info, log_success, log_warn,
};
use anyhow::Context;

pub fn remote_update(
    _args: &Args,
    registry: &mut Registry,
    id: &Option<String>,
    name: &Option<String>,
    provider: &Option<String>,
) -> anyhow::Result<()> {
    if registry.remotes.is_empty() {
        log_warn!("no remotes configured");
        return Ok(());
    }

    let remote_info = match id {
        Some(value) => {
            if !registry.remotes.iter().any(|r| r.id == *value) {
                anyhow::bail!("remote with id '{}' not found", value);
            }

            remote::Utils::remote_by_id(registry, value).context("remote not found")?
        }
        None => remote::Prompt::remote::<fn(inquire::Select<String>) -> inquire::Select<String>>(
            "Select a remote to update:",
            registry,
            None,
        )
        .context("failed to execute prompt")?,
    };

    let name = match name {
        Some(value) => value,
        None => &remote::Prompt::name()
            .with_default(&remote_info.remote_name)
            .prompt()
            .context("failed to execute prompt")?
            .clone(),
    };

    let provider = match provider {
        Some(value) => value,
        None => &remote::Prompt::provider()
            .with_default(&remote_info.provider)
            .prompt()
            .context("failed to create thext prompt")?
            .clone(),
    };

    registry
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
