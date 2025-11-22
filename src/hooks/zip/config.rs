use crate::{
    config::prelude::{HookConfig, HookExecType, Hooks},
    hooks::zip::hook::ZipHookConfig,
    log_info,
};
use anyhow::Context;
use inquire::Text;

impl ZipHookConfig {
    pub fn build(exec_type: HookExecType) -> anyhow::Result<HookConfig> {
        log_info!("configuring {} for {}", Hooks::Zip, exec_type);

        match exec_type {
            HookExecType::Push => {
                let level = Text::new("Compression level (0-9):")
                    .with_default("9")
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
