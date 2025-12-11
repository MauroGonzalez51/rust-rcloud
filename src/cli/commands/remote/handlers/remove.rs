use crate::{
    cli::{commands::remote::utils::remote, context::CommandContext},
    log_info, log_success, log_warn,
};
use anyhow::Context;

#[derive(Clone)]
pub struct LocalArgs<'a> {
    pub id: &'a Option<String>,
}

pub fn remote_remove(mut context: CommandContext<LocalArgs>) -> anyhow::Result<()> {
    if context.registry.remotes.is_empty() {
        log_warn!("no remotes configured");
        return Ok(());
    }

    let remote = match context.local.id {
        Some(value) => {
            if !context.registry.remotes.iter().any(|r| r.id == *value) {
                anyhow::bail!("remote with '{}' not found", value);
            }

            remote::Utils::remote_by_id(&context.registry, value).context("remote not found")?
        }
        None => remote::Prompt::remote::<fn(inquire::Select<String>) -> inquire::Select<String>>(
            "Select a remote to remove:",
            &context.registry,
            None,
        )
        .context("failed to execute prompt")?,
    };

    log_info!(
        "removing remote: {} ({})",
        remote.remote_name,
        remote.provider
    );

    context
        .registry
        .tx(|rgx| {
            rgx.remotes.retain(|r| r.id != remote.id);
        })
        .context("failed to execute transaction")?;

    log_success!("remote removed successfully");

    Ok(())
}
