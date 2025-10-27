use crate::{
    cli::{commands::remote::utils::remote, parser::Args},
    config::prelude::Registry,
    log_info, log_success, log_warn,
};
use anyhow::Context;

pub fn remote_remove(
    _args: &Args,
    registry: &mut Registry,
    id: &Option<String>,
) -> anyhow::Result<()> {
    if registry.remotes.is_empty() {
        log_warn!("no remotes configured");
        return Ok(());
    }

    let remote = match id {
        Some(value) => {
            if !registry.remotes.iter().any(|r| r.id == *value) {
                anyhow::bail!("remote with '{}' not found", value);
            }

            remote::Utils::remote_by_id(registry, value).context("remote not found")?
        }
        None => remote::Prompt::remote::<fn(inquire::Select<String>) -> inquire::Select<String>>(
            registry, None,
        )
        .context("failed to execute prompt")?,
    };

    log_info!(
        "removing remote: {} ({})",
        remote.remote_name,
        remote.provider
    );

    registry
        .tx(|rgx| {
            rgx.remotes.retain(|r| r.id != remote.id);
        })
        .context("failed to execute transaction")?;

    log_success!("remote removed successfully");

    Ok(())
}
