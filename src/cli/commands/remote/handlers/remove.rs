use crate::{
    cli::{
        commands::remote::utils::remote, context::CommandContext, output::OutputSink,
        prompter::Prompter,
    },
    log_warn,
};
use anyhow::Context;

#[derive(Clone)]
pub struct LocalArgs<'a> {
    pub id: &'a Option<String>,
}

impl<'a> Default for LocalArgs<'a> {
    fn default() -> Self {
        Self { id: &None }
    }
}

pub fn remote_remove<P: Prompter, S: OutputSink>(
    context: CommandContext<LocalArgs>,
    prompter: &P,
    sink: &S,
) -> anyhow::Result<()> {
    if context.with_registry()?.remotes.is_empty() {
        log_warn!("no remotes configured");
        sink.warn("no remotes configured");
        return Ok(());
    }

    let remote = match context.local.id {
        Some(value) => {
            if !context
                .with_registry()?
                .remotes
                .iter()
                .any(|r| r.id == *value)
            {
                anyhow::bail!("remote with '{}' not found", value);
            }

            remote::Utils::remote_by_id(std::sync::Arc::clone(&context.registry), value)
                .context("remote not found")?
        }
        None => remote::Prompt::remote(
            prompter,
            "Select a remote to remove:",
            std::sync::Arc::clone(&context.registry),
        )
        .context("failed to execute prompt")?,
    };

    sink.info(format!(
        "removing remote: {} ({})",
        remote.remote_name, remote.provider
    ));

    context
        .with_registry()?
        .tx(|rgx| {
            rgx.remotes.retain(|r| r.id != remote.id);
        })
        .context("failed to execute transaction")?;

    sink.success("remote removed successfully");

    Ok(())
}
