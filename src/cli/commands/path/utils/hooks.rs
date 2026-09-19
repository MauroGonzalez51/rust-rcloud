use crate::{
    cli::prompter::Prompter,
    config::prelude::{HookConfig, HookExecType, Hooks},
    hooks::prelude::HookBuilder,
};
use anyhow::Context;

#[derive(Clone)]
struct ExecOption {
    exec_type: HookExecType,
    description: String,
}

impl std::fmt::Display for ExecOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:<4} - {}", self.exec_type, self.description)
    }
}

pub fn declare_hooks<P: Prompter>(
    prompter: &P,
) -> anyhow::Result<(Vec<HookConfig>, Vec<HookConfig>)> {
    let mut push_hooks = Vec::<HookConfig>::new();
    let mut pull_hooks = Vec::<HookConfig>::new();

    let add_hooks = prompter
        .confirm("would you like to add some hooks?", false)
        .context("failed to create confirm prompt")?;

    if add_hooks {
        loop {
            let hook_type = prompter
                .select(
                    "Select a Hook:",
                    vec![Hooks::Zip, Hooks::Backup, Hooks::Encryption],
                )
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

            let selected_options = prompter
                .multi_select("Select when the Hook will run:", options, &[])
                .context("failed to select hook exec type")?;

            let should_share_config = selected_options.len() == 2 && hook_type.share_config();

            match should_share_config {
                true => {
                    let (push_config, pull_config) = HookBuilder::new(hook_type, None)
                        .build_shared(prompter)
                        .ok_or_else(|| anyhow::anyhow!("operation not supported"))??;

                    push_hooks.push(push_config);
                    pull_hooks.push(pull_config);
                }
                false => {
                    for option in selected_options {
                        let hook_config = HookBuilder::new(hook_type, Some(option.exec_type))
                            .build(prompter)?;

                        match option.exec_type {
                            HookExecType::Push => push_hooks.push(hook_config),
                            HookExecType::Pull => pull_hooks.push(hook_config),
                        }
                    }
                }
            }

            let add_another = prompter
                .confirm("Add another hook?", false)
                .context("failed to get confirmation")?;

            if !add_another {
                break;
            }
        }
    }

    Ok((push_hooks, pull_hooks))
}
