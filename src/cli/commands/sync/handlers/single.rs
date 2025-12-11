use crate::{
    cli::{
        commands::{path::utils::path, sync::utils},
        context::CommandContext,
    },
    config::prelude::HookExecType,
    log_warn,
};

use anyhow::Context;

#[derive(Clone)]
pub struct LocalArgs<'a> {
    pub direction: &'a Option<HookExecType>,
    pub path_id: &'a Option<String>,
    pub force: &'a bool,
    pub clean: &'a bool,
}

// TODO: force and clean should be asked as well
impl<'a> Default for LocalArgs<'a> {
    fn default() -> Self {
        Self {
            direction: &None,
            path_id: &None,
            force: &false,
            clean: &true,
        }
    }
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
        .registry
        .paths
        .iter()
        .find(|p| p.id == path_id)
        .ok_or_else(|| anyhow::anyhow!("path does not exists"))?
        .clone();

    let remote_config = context
        .registry
        .remotes
        .iter()
        .find(|r| r.id == path_config.remote_id)
        .ok_or_else(|| anyhow::anyhow!("remote does not exists"))?
        .clone();

    let hooks = &path_config.hooks;

    match direction {
        HookExecType::Push => utils::push(utils::push::PushOptions {
            config: &context.config,
            registry: &mut context.registry,
            paths: utils::push::PushOptionsPaths {
                rclone: &context.global.rclone,
                remote: &remote_config,
                path_config: &path_config,
            },
            hooks: &hooks.push,
            force: context.local.force,
        })?,

        HookExecType::Pull => utils::pull(utils::pull::PullOptions {
            config: &context.config,
            registry: &mut context.registry,
            paths: utils::pull::PullOptionsPaths {
                rclone: &context.global.rclone,
                remote: &remote_config,
                path_config: &path_config,
            },
            hooks: &hooks.pull,
            clean: context.local.clean,
            force: context.local.force,
        })?,
    }

    Ok(context)
}
