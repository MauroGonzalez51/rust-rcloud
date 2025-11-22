use crate::{
    config::prelude::{Hook, HookContext, HookExecType, Hooks},
    define_hook,
};
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
