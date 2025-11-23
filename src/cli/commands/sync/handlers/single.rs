use crate::{
    cli::{
        commands::{path::utils::path, sync::utils},
        context::CommandContext,
    },
    config::prelude::HookExecType,
    log_warn,
};
use anyhow::Context;

pub struct LocalArgs<'a> {
    pub direction: &'a Option<HookExecType>,
    pub path_id: &'a Option<String>,
    pub force: &'a bool,
    pub clean: &'a bool,
}

pub fn sync_single(
    mut context: CommandContext<LocalArgs>,
) -> anyhow::Result<CommandContext<LocalArgs>> {
    if *context.local.force {
        log_warn!("using --force");
    }

    let direction = match context.local.direction {
        Some(value) => value,
        None => &HookExecType::select("Select direction:")
            .with_vim_mode(true)
            .prompt()
            .context("failed to select direction")?,
    };

    let path_id = match context.local.path_id {
        Some(value) => value.clone(),
        None => path::Prompt::path_config("Select the path to sync:", &context.registry)
            .context("failed to select path")?
            .clone(),
    };

    let path_config = context
        .paths
        .iter()
        .find(|p| p.id == path_id)
        .ok_or_else(|| anyhow::anyhow!("path does not exists"))?
        .clone();

    let remote_config = context
        .remotes
        .iter()
        .find(|r| r.id == path_config.remote_id)
        .ok_or_else(|| anyhow::anyhow!("remote does not exists"))?
        .clone();

    let hooks = &path_config.hooks;

    utils::clean(direction, context.local.clean, &path_config.local_path)?;

    match direction {
        HookExecType::Push => utils::push(
            &mut context.registry,
            &context.global.rclone,
            &remote_config,
            &path_config,
            &hooks.push,
            context.local.force,
        )?,

        HookExecType::Pull => utils::pull(
            &mut context.registry,
            &context.global.rclone,
            &remote_config,
            &path_config,
            &hooks.pull,
            context.local.force,
        )?,
    }

    Ok(context)
}
