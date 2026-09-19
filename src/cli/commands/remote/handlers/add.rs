use crate::{
    cli::{
        commands::remote::utils::remote, context::CommandContext, output::OutputSink,
        prompter::Prompter,
    },
    config::prelude::*,
    log_debug,
};
use anyhow::Context;
use uuid::Uuid;

#[derive(Clone)]
pub struct LocalArgs<'a> {
    pub name: &'a Option<String>,
    pub provider: &'a Option<String>,
}

impl<'a> Default for LocalArgs<'a> {
    fn default() -> Self {
        Self {
            name: &None,
            provider: &None,
        }
    }
}

pub fn remote_add<P: Prompter, S: OutputSink>(
    context: CommandContext<LocalArgs>,
    prompter: &P,
    sink: &S,
) -> anyhow::Result<()> {
    let remote_name = match context.local.name {
        Some(value) => value.clone(),
        None => remote::Prompt::name(prompter).context("failed to create prompt")?,
    };

    let provider = match context.local.provider {
        Some(value) => value.clone(),
        None => remote::Prompt::provider(prompter).context("failed to create prompt")?,
    };

    log_debug!("adding remote '{remote_name}' ({provider}) to registry");

    context
        .with_registry()?
        .tx(|rgx| {
            rgx.remotes.push(Remote {
                id: Uuid::new_v4().to_string(),
                remote_name: remote_name.clone(),
                provider: provider.clone(),
            })
        })
        .context("error inside transaction")?;

    sink.success("remote added succesfully");

    Ok(())
}
