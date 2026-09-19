use crate::{
    cli::prompter::{Prompter, TextOptions},
    config::prelude::{HookConfig, HookExecType, Hooks},
    hooks::prelude::{HookBuilderTrait, ZipHookConfig},
    log_info,
};
use anyhow::Context;

impl HookBuilderTrait for ZipHookConfig {
    fn build<P: Prompter>(exec: HookExecType, prompter: &P) -> anyhow::Result<HookConfig> {
        log_info!("configuring {} for {}", Hooks::Zip, exec);

        match exec {
            HookExecType::Push => {
                let level = prompter
                    .text(
                        "Compresion level (0-9):",
                        TextOptions::new().default_value("9"),
                    )
                    .context("failed to get compression level")?
                    .parse::<i64>()
                    .context("failed to parse compresion level")?;

                let exclude = prompter
                    .text(
                        "Exclude patterns:",
                        TextOptions::new().help("comma-separated, glob only, optional"),
                    )
                    .context("failed to get exclude patterns")?;

                let exclude = match exclude.trim().is_empty() {
                    true => None,
                    false => Some(
                        exclude
                            .split(',')
                            .map(|p| p.trim().to_string())
                            .filter(|p| !p.is_empty())
                            .collect(),
                    ),
                };

                Ok(HookConfig::Zip(Self {
                    exec: HookExecType::Push,
                    level: Some(level),
                    exclude,
                }))
            }
            HookExecType::Pull => Ok(HookConfig::Zip(Self {
                exec: HookExecType::Pull,
                level: None,
                exclude: None,
            })),
        }
    }
}
