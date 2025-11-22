use crate::{
    config::prelude::{Hook, HookConfig, HookContext, HookExecType, Hooks},
    define_hook, log_info,
};
use anyhow::Context;
use inquire_derive::Selectable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Copy, Selectable)]
pub enum BackupType {
    Local,
    Remote,
}

impl std::fmt::Display for BackupType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackupType::Local => write!(f, "Local"),
            BackupType::Remote => write!(f, "Remote"),
        }
    }
}

define_hook!(BackupHook {
    types: Vec<BackupType>,
    destination: String,
    replicas: u32,
});

impl BackupHook {}

impl Hook for BackupHook {
    fn name(&self) -> &'static str {
        "backup"
    }
    fn exec_type(&self) -> &HookExecType {
        &self.exec
    }
    fn hook_type(&self) -> &Hooks {
        &Hooks::Backup
    }

    fn process(&self, ctx: HookContext) -> anyhow::Result<HookContext> {
        Ok(ctx)
    }
}

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
