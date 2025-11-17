use crate::{
    cli::{
        commands::{path::utils::path, sync::utils},
        parser::Args,
    },
    config::prelude::{HookExecType, Registry},
    log_warn,
};
use anyhow::Context;

pub fn path_sync(
    args: &Args,
    registry: &mut Registry,
    direction: &Option<HookExecType>,
    path_id: &Option<String>,
    force: &bool,
    clean: &bool,
) -> anyhow::Result<()> {
    if *force {
        log_warn!("using --force");
    }

    let direction = match direction {
        Some(value) => value,
        None => &HookExecType::select("Select direction:")
            .with_vim_mode(true)
            .prompt()
            .context("failed to select direction")?,
    };

    let path_id = match path_id {
        Some(value) => value.clone(),
        None => path::Prompt::path_config("Select the path to sync:", registry)
            .context("failed to select path")?
            .clone(),
    };

    let path_config = registry
        .paths
        .iter()
        .find(|p| p.id == path_id)
        .ok_or_else(|| anyhow::anyhow!("path does not exists"))?
        .clone();

    let remote_config = registry
        .remotes
        .iter()
        .find(|r| r.id == path_config.remote_id)
        .ok_or_else(|| anyhow::anyhow!("remote does not exists"))?
        .clone();

    let hooks = &path_config.hooks;

    utils::options::clean(direction, clean, &path_config.local_path)?;

    match direction {
        HookExecType::Push => utils::push::push(
            args,
            registry,
            &remote_config,
            &path_config,
            &hooks.push,
            force,
        )?,

        HookExecType::Pull => utils::pull::pull(
            args,
            registry,
            &remote_config,
            &path_config,
            &hooks.pull,
            force,
        )?,
    }

    Ok(())
}
