use crate::{
    cli::{commands::path::utils::path, parser::Args},
    config::{hooks::zip::ZipHookConfig, prelude::*},
    log_info, log_warn,
};
use anyhow::Context;
use inquire::{Confirm, Text};

pub fn compute_hook_output(source: &str, hook_type: Hooks) -> String {
    match hook_type {
        Hooks::Zip => format!("{}.zip", source),
    }
}

pub fn get_next_source_push(hooks: &[HookConfig], local_path: &str) -> String {
    if hooks.is_empty() {
        return local_path.to_string();
    }

    let last_hook = &hooks[hooks.len() - 1];
    compute_hook_output(last_hook.source(), *last_hook.hook_type())
}

pub fn get_next_source_pull(hooks: &[HookConfig], remote_path: &str) -> String {
    if hooks.is_empty() {
        return remote_path.to_string();
    }

    let first_hook = &hooks[0];
    compute_hook_output(first_hook.source(), *first_hook.hook_type())
}

pub fn build_hook_push(hook_type: &Hooks, source: &str) -> anyhow::Result<HookConfig> {
    log_info!("configuring {} for {}", hook_type, HookExecType::Push);

    match hook_type {
        Hooks::Zip => {
            let level = Text::new("Compression level (0-9):")
                .with_default("6")
                .prompt()
                .context("failed to get compression level")?
                .parse::<i64>()
                .context("failed to parse compresion level")?;

            let exclude = Text::new("Exclude patterns: ")
                .with_help_message("comma-separated, glob only, optional")
                .prompt_skippable()
                .context("failed to get exclude patterns")?;

            let exclude = exclude.map(|s| {
                s.split(',')
                    .map(|p| p.trim().to_string())
                    .filter(|p| !p.is_empty())
                    .collect()
            });

            Ok(HookConfig::Zip(ZipHookConfig {
                exec: HookExecType::Push,
                source: source.to_string(),
                level: Some(level),
                exclude,
            }))
        }
    }
}

pub fn build_hook_pull(hook_type: &Hooks, source: &str) -> anyhow::Result<HookConfig> {
    log_info!("configuring {} for {}", hook_type, HookExecType::Pull);

    match hook_type {
        Hooks::Zip => Ok(HookConfig::Zip(ZipHookConfig {
            exec: HookExecType::Pull,
            source: source.to_string(),
            level: None,
            exclude: None,
        })),
    }
}

pub fn path_add(
    _args: &Args,
    registry: &mut Registry,
    remote_id: &Option<String>,
    local_path: &Option<String>,
    remote_path: &Option<String>,
) -> anyhow::Result<()> {
    if registry.remotes.is_empty() {
        log_warn!("there are no remotes configured");
        return Ok(());
    }

    let remote_id = match remote_id {
        Some(value) => value,
        None => &path::Prompt::remote_id::<
            fn(inquire::Select<'_, String>) -> inquire::Select<'_, String>,
        >(registry, None)
        .context("failed to get remote_id")?,
    };

    let local_path = match local_path {
        Some(value) => value,
        None => &path::Prompt::path("local path:")
            .prompt()
            .context("failed to get local path")?,
    };

    let remote_path = match remote_path {
        Some(value) => value,
        None => &path::Prompt::path("remote path")
            .prompt()
            .context("failed to get remote path")?,
    };

    let mut push_hooks = Vec::<HookConfig>::new();
    let mut pull_hooks = Vec::<HookConfig>::new();

    let add_hooks = Confirm::new("would you like to add some hooks?")
        .prompt()
        .context("failed to create confirm prompt")?;

    if add_hooks {
        loop {
            let hook_type = Hooks::select("Select a Hook:")
                .prompt()
                .context("failed to select hook")?;

            let hook_exec_type = HookExecType::select("Select when the Hook will run:")
                .prompt()
                .context("failed to select hook exec type")?;

            match hook_exec_type {
                HookExecType::Push => {
                    let source = get_next_source_push(&push_hooks, local_path);

                    let hook =
                        build_hook_push(&hook_type, &source).context("failed to build hook")?;

                    push_hooks.push(hook);
                }
                HookExecType::Pull => {
                    let source = get_next_source_pull(&push_hooks, local_path);

                    let hook =
                        build_hook_pull(&hook_type, &source).context("failed to build hook")?;

                    pull_hooks.insert(0, hook);
                }
                HookExecType::Both => {
                    let push_source = get_next_source_push(&push_hooks, local_path);
                    let pull_source = get_next_source_pull(&pull_hooks, remote_path);

                    let push_hook = build_hook_push(&hook_type, &push_source)
                        .context("failed to build push hook")?;

                    let pull_hook = build_hook_pull(&hook_type, &pull_source)
                        .context("failed to build pull hook")?;

                    push_hooks.push(push_hook);
                    pull_hooks.insert(0, pull_hook);
                }
            }

            let add_another = Confirm::new("Add another hook?")
                .with_default(false)
                .prompt()
                .context("failed to get confirmation")?;

            if !add_another {
                break;
            }
        }
    }

    let path_config = PathConfig {
        id: uuid::Uuid::new_v4().to_string(),
        remote_id: remote_id.clone(),
        local_path: local_path.clone(),
        remote_path: remote_path.clone(),
        hooks: PathConfigHooks {
            push: push_hooks,
            pull: pull_hooks,
        },
    };

    let confirm_save = Confirm::new("Save this configuration?")
        .with_default(true)
        .prompt()
        .context("failed to get confirmation")?;

    if confirm_save {
        registry
            .tx(|rgx| {
                rgx.paths.push(path_config);
            })
            .context("failed to execute transaction")?;
    };

    Ok(())
}
