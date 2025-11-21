use crate::{
    cli::{commands::remote::utils::remote, context::CommandContext},
    log_debug, log_info, log_success, log_warn,
};
use anyhow::Context;

pub struct LocalArgs<'a> {
    pub id: &'a Option<String>,
    pub name: &'a Option<String>,
    pub provider: &'a Option<String>,
}

pub fn remote_update(mut context: CommandContext<LocalArgs>) -> anyhow::Result<()> {
    if context.remotes.is_empty() {
        log_warn!("no remotes configured");
        return Ok(());
    }

    let remote_info = match context.local.id {
        Some(value) => {
            if !context.remotes.iter().any(|r| r.id == *value) {
                anyhow::bail!("remote with id '{}' not found", value);
            }

            remote::Utils::remote_by_id(&context.registry, value).context("remote not found")?
        }
        None => remote::Prompt::remote::<fn(inquire::Select<String>) -> inquire::Select<String>>(
            "Select a remote to update:",
            &context.registry,
            None,
        )
        .context("failed to execute prompt")?,
    };

    log_debug!("using remote_info: {:?}", remote_info);

    let name = match context.local.name {
        Some(value) => value,
        None => &remote::Prompt::name()
            .with_default(&remote_info.remote_name)
            .prompt()
            .context("failed to execute prompt")?
            .clone(),
    };

    let provider = match context.local.provider {
        Some(value) => value,
        None => &remote::Prompt::provider()
            .with_default(&remote_info.provider)
            .prompt()
            .context("failed to create thext prompt")?
            .clone(),
    };

    context
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
