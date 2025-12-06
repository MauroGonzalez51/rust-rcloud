use crate::{
    config::prelude::{HookConfig, HookExecType, Hooks},
    hooks::prelude::HookBuilder,
};
use anyhow::Context;
use inquire::{Confirm, MultiSelect};

struct ExecOption {
    exec_type: HookExecType,
    description: String,
}

impl std::fmt::Display for ExecOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:<4} - {}", self.exec_type, self.description)
    }
}

pub fn declare_hooks() -> anyhow::Result<(Vec<HookConfig>, Vec<HookConfig>)> {
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

            let options = vec![
                ExecOption {
                    exec_type: HookExecType::Push,
                    description: hook_type.describe(HookExecType::Push).to_string(),
                },
                ExecOption {
                    exec_type: HookExecType::Pull,
                    description: hook_type.describe(HookExecType::Pull).to_string(),
                },
            ];

            let selected_options = MultiSelect::new("Select when the Hook will run:", options)
                .prompt()
                .context("failed to select hook exec type")?;

            for option in selected_options {
                let exec_type = option.exec_type;

                match exec_type {
                    HookExecType::Push => {
                        push_hooks.push(
                            HookBuilder::new()
                                .with_hook_type(hook_type)
                                .with_exec_type(exec_type)
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

    Ok((push_hooks, pull_hooks))
}
