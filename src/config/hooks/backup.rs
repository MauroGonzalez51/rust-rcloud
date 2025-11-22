use crate::{
    config::prelude::{Hook, HookConfig, HookContext, HookExecType, Hooks},
    define_hook, log_info,
};
use anyhow::Context;
use std::path::PathBuf;

define_hook!(BackupHook {
    source: String,
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
        let ctx = HookContext::new(PathBuf::new(), &ctx.rclone_path, &ctx.remote_config);

        log_info!("context: {:?} | Self: {:?}", ctx, self);

        Ok(ctx)
    }
}

impl BackupHookConfig {
    pub fn build(exec_type: HookExecType, source: &str) -> anyhow::Result<HookConfig> {
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
            source: source.to_string(),
            destination,
            replicas,
        }))
    }
}
