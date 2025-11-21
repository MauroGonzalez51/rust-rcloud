use crate::{
    config::prelude::{Hook, HookContext, HookExecType, Hooks},
    define_hook,
};
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

    fn process(&self, _ctx: crate::HookContext) -> anyhow::Result<HookContext> {
        let ctx = HookContext::new(PathBuf::new());

        Ok(ctx)
    }
}

impl BackupHookConfig {
    pub fn build() {}
}
