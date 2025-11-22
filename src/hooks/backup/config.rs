use crate::{
    config::prelude::{HookConfig, HookExecType, Hooks},
    hooks::backup::{BackupHookConfig, BackupType},
    log_info,
};
use anyhow::Context;

impl BackupHookConfig {
    pub fn build(exec_type: HookExecType) -> anyhow::Result<HookConfig> {
        log_info!("configuring {} for {}", Hooks::Backup, exec_type);

        let destination = inquire::Text::new("Remote backup destination:")
            .prompt()
            .context("invalid destination")?;

        let replicas = inquire::Text::new("Max replicas:")
            .with_default("1")
            .prompt()
            .context("invalid replicas")?
            .parse::<u32>()
            .context("not a number")?;

        let types = BackupType::multi_select("Select backup type(s):")
            .prompt()
            .context("failed to select backup types")?;

        Ok(HookConfig::Backup(Self {
            exec: exec_type,
            types,
            destination,
            replicas,
        }))
    }
}
