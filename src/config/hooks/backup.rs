use crate::{
    config::prelude::{Hook, HookConfig, HookContext, HookExecType, Hooks},
    define_hook, log_info,
};
use anyhow::Context;

define_hook!(BackupHook {
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

        Ok(HookConfig::Backup(Self {
            exec: exec_type,
            destination,
            replicas,
        }))
    }
}
