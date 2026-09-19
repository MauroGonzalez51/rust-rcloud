use crate::{
    cli::{
        commands::{path::utils::path, sync::utils},
        context::CommandContext,
        output::OutputSink,
        prompter::Prompter,
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

pub fn sync_single<'a, P: Prompter, S: OutputSink>(
    context: CommandContext<LocalArgs<'a>>,
    prompter: &P,
    sink: &S,
) -> anyhow::Result<CommandContext<LocalArgs<'a>>> {
    let direction = match context.local.direction {
        Some(value) => *value,
        None => prompter
            .select(
                "Select direction:",
                vec![HookExecType::Push, HookExecType::Pull],
            )
            .context("failed to select direction")?,
    };

    let path_id = match context.local.path_id {
        Some(value) => value.clone(),
        None => path::Prompt::path_config(
            prompter,
            "Select the path to sync:",
            std::sync::Arc::clone(&context.registry),
        )
        .context("failed to select path")?,
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
        Some(value) => *value,
        None => prompter
            .confirm("Should we use force option?", false)
            .context("failed to prompt user")?,
    };

    let clean = match context.local.clean {
        Some(value) => *value,
        None => prompter
            .confirm("Should we use clean option?", true)
            .context("failed to prompt user")?,
    };

    match direction {
        HookExecType::Push => {
            utils::push(utils::push::PushOptions {
                config: &context.config,
                registry: std::sync::Arc::clone(&context.registry),
                paths: utils::push::PushOptionsPaths {
                    rclone: &context.global.rclone,
                    remote: &remote_config,
                    path_config: &path_config,
                },
                hooks: &hooks.push,
                force: &force,
            })?;
            sink.success(format!(
                "pushed {} -> {}:{}",
                path_config.local_path, remote_config.remote_name, path_config.remote_path
            ));
        }

        HookExecType::Pull => {
            utils::pull(utils::pull::PullOptions {
                config: &context.config,
                registry: std::sync::Arc::clone(&context.registry),
                paths: utils::pull::PullOptionsPaths {
                    rclone: &context.global.rclone,
                    remote: &remote_config,
                    path_config: &path_config,
                },
                hooks: &hooks.pull,
                clean: &clean,
                force: &force,
            })?;
            sink.success(format!(
                "pulled {}:{} -> {}",
                remote_config.remote_name, path_config.remote_path, path_config.local_path
            ));
        }
    }

    Ok(context)
}
