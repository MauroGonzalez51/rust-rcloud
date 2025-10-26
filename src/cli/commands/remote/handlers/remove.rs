use crate::{cli::parser::Args, config::prelude::Registry, log_info, log_success, log_warn};
use anyhow::Context;
use inquire::Select;

pub fn remote_remove(
    _args: &Args,
    registry: &mut Registry,
    id: &Option<String>,
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

    log_info!(
        "removing remote: {} ({})",
        remote_info.remote_name,
        remote_info.provider
    );

    registry
        .tx(|rgx| {
            rgx.remotes.retain(|r| r.id != *id);
        })
        .context("failed to execute transaction")?;

    log_success!("remote removed succesfully");

    Ok(())
}
