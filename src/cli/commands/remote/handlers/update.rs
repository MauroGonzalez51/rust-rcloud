use crate::{cli::parser::Args, config::prelude::Registry, log_info, log_success, log_warn};
use anyhow::Context;
use inquire::{Select, Text};

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

    let id = match id {
        Some(value) => value,
        None => {
            let options: Vec<(String, String)> = registry
                .remotes
                .iter()
                .map(|r| {
                    (
                        format!("{} ({}) [{}]", r.remote_name, r.provider, &r.id[..8]),
                        r.id.clone(),
                    )
                })
                .collect();

            let display_options = options.iter().map(|(display, _)| display.clone()).collect();

            let selected = Select::new("Select remote to remove", display_options)
                .with_vim_mode(true)
                .with_page_size(10)
                .with_help_message("<remote_name> (<remote_provider>) [<...remote_id>]")
                .prompt()
                .context("failed to select remote")?;

            &options
                .into_iter()
                .find(|(display, _)| *display == selected)
                .map(|(_, id)| id)
                .ok_or_else(|| anyhow::anyhow!("failed to find selected remote"))?
        }
    };

    let remote_info = registry
        .remotes
        .iter()
        .find(|r| r.id == *id)
        .ok_or_else(|| anyhow::anyhow!("remote with id '{}' not found", id))?;

    let name = match name {
        Some(value) => value,
        None => &Text::new("Provide the new remote name:")
            .with_default(&remote_info.remote_name)
            .prompt()
            .context("failed to create text prompt")?
            .clone(),
    };

    let provider = match provider {
        Some(value) => value,
        None => &Text::new("Provide the new remote provider:")
            .with_default(&remote_info.provider)
            .prompt()
            .context("failed to create thext prompt")?
            .clone(),
    };

    registry
        .tx(|rgx| {
            if let Some(remote) = rgx.remotes.iter_mut().find(|r| r.id == *id) {
                log_info!("found remote to update");
                remote.remote_name = name.clone();
                remote.provider = provider.clone();
            }
        })
        .context("failed to execute transaction")?;

    log_success!("remote updated succesfully");

    Ok(())
}
