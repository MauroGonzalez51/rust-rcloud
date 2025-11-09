use crate::{
    cli::{commands::path::utils::path, parser::Args},
    config::prelude::*,
    log_debug, log_warn,
};
use anyhow::Context;
use inquire::Confirm;

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

    let local_path = std::fs::canonicalize(shellexpand::tilde(&local_path).to_string())
        .with_context(|| format!("failed to resolve local path: {}", local_path))?
        .to_string_lossy()
        .to_string();

    let remote_path = match remote_path {
        Some(value) => value,
        None => &path::Prompt::path("remote path:")
            .prompt()
            .context("failed to get remote path")?,
    };

    log_debug!(
        "{} -> (remote_id: {}):{}",
        local_path,
        remote_id,
        remote_path
    );

    let mut push_hooks = Vec::<HookConfig>::new();
    let mut pull_hooks = Vec::<HookConfig>::new();

    let add_hooks = Confirm::new("would you like to add some hooks?")
        .with_default(false)
        .prompt()
        .context("failed to create confirm prompt")?;

    if add_hooks {
        loop {
            let hook_type = Hooks::select("Select a Hook:")
                .prompt()
                .context("failed to select hook")?;

            let hook_exec_type = HookExecType::multi_select("Select when the Hook will run:")
                .prompt()
                .context("failed to select hook exec type")?;

            for exec_type in hook_exec_type {
                match exec_type {
                    HookExecType::Push => {
                        push_hooks.push(
                            HookBuilder::new()
                                .with_hook_type(hook_type)
                                .with_exec_type(exec_type)
                                .with_paths(local_path.clone(), remote_path.clone())
                                .with_list(&push_hooks)
                                .build()
                                .context("failed to build push hook")?,
                        );
                    }
                    HookExecType::Pull => {
                        pull_hooks.insert(
                            0,
                            HookBuilder::new()
                                .with_hook_type(hook_type)
                                .with_exec_type(exec_type)
                                .with_paths(local_path.clone(), remote_path.clone())
                                .with_list(&pull_hooks)
                                .build()
                                .context("failed to build pull hook")?,
                        );
                    }
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

    let add_tags = Confirm::new("Add some tags?")
        .with_default(false)
        .prompt()
        .context("failed to get confirmation")?;

    let mut tags: Vec<String> = vec![];

    if add_tags {
        tags = TagOption::multiple_select("Select tags:", registry)
            .context("failed to select tags")?;
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
        tags,
    };

    log_debug!("using path_config: {:?}", path_config);

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
    }

    Ok(())
}
