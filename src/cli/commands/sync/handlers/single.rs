use crate::{
    cli::{
        commands::{path::utils::path, sync::utils},
        context::CommandContext,
    },
    config::prelude::HookExecType,
    log_info, log_warn,
};

use anyhow::Context;

#[derive(Clone, Debug)]
pub struct LocalArgs<'a> {
    pub direction: &'a Option<HookExecType>,
    pub path_id: &'a Option<String>,
    pub force: &'a Option<bool>,
    pub clean: &'a Option<bool>,
}

impl<'a> Default for LocalArgs<'a> {
    fn default() -> Self {
        Self {
            direction: &None,
            path_id: &None,
            force: &None,
            clean: &None,
        }
    }
}

pub fn sync_single(
    context: CommandContext<LocalArgs>,
) -> anyhow::Result<CommandContext<LocalArgs>> {
    let direction = match context.local.direction {
        Some(value) => value,
        None => &HookExecType::select("Select direction:")
            .with_vim_mode(true)
            .prompt()
            .context("failed to select direction")?,
    };

    let path_id = match context.local.path_id {
        Some(value) => value.clone(),
        None => path::Prompt::path_config(
            "Select the path to sync:",
            std::sync::Arc::clone(&context.registry),
        )
        .context("failed to select path")?
        .clone(),
    };

    let path_config = context
        .with_registry()?
        .paths
        .iter()
        .find(|p| p.id == path_id)
        .ok_or_else(|| anyhow::anyhow!("path does not exists"))?
        .clone();

    let remote_config = context
        .with_registry()?
        .remotes
        .iter()
        .find(|r| r.id == path_config.remote_id)
        .ok_or_else(|| anyhow::anyhow!("remote does not exists"))?
        .clone();

    let hooks = &path_config.hooks;

    if let Some(force) = context.local.force
        && *force
    {
        log_warn!("using force option");
    }

    if let Some(clean) = context.local.clean
        && *clean
    {
        log_info!(
            "local directory {} will be cleaned when using {}",
            path_config.local_path,
            HookExecType::Pull
        );
    }

    let force = match context.local.force {
        Some(value) => value,
        None => &inquire::Confirm::new("Should we use force option?")
            .with_default(false)
            .prompt()
            .context("failed to prompt user")?,
    };

    let clean = match context.local.clean {
        Some(value) => value,
        None => &inquire::Confirm::new("Should we use clean option?")
            .with_default(true)
            .prompt()
            .context("failed to prompt user")?,
    };

    match direction {
        HookExecType::Push => utils::push(utils::push::PushOptions {
            config: &context.config,
            registry: std::sync::Arc::clone(&context.registry),
            paths: utils::push::PushOptionsPaths {
                rclone: &context.global.rclone,
                remote: &remote_config,
                path_config: &path_config,
            },
            hooks: &hooks.push,
            force,
        })?,

        HookExecType::Pull => utils::pull(utils::pull::PullOptions {
            config: &context.config,
            registry: std::sync::Arc::clone(&context.registry),
            paths: utils::pull::PullOptionsPaths {
                rclone: &context.global.rclone,
                remote: &remote_config,
                path_config: &path_config,
            },
            hooks: &hooks.pull,
            clean,
            force,
        })?,
    }

    Ok(context)
}
