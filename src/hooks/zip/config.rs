use crate::{
    config::prelude::{HookConfig, HookExecType, Hooks},
    hooks::prelude::{HookBuilderTrait, ZipHookConfig},
    log_info,
};
use anyhow::Context;
use inquire::Text;

impl HookBuilderTrait for ZipHookConfig {
    fn build(exec: HookExecType) -> anyhow::Result<HookConfig> {
        log_info!("configuring {} for {}", Hooks::Zip, exec);

        match exec {
            HookExecType::Push => {
                let level = Self::level()
                    .prompt()
                    .context("failed to get compression level")?
                    .parse::<i64>()
                    .context("failed to parse compresion level")?;

                let exclude = Self::exclude()
                    .prompt_skippable()
                    .context("failed to get exclude patterns")?;

                let exclude = exclude.map(|s| {
                    s.split(',')
                        .map(|p| p.trim().to_string())
                        .filter(|p| !p.is_empty())
                        .collect()
                });

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

impl ZipHookConfig {
    fn level() -> Text<'static, 'static> {
        Text::new("Compresion level (0-9):").with_default("9")
    }

    fn exclude() -> Text<'static, 'static> {
        Text::new("Exclude patterns:").with_help_message("comma-separated, glob only, optional")
    }
}
