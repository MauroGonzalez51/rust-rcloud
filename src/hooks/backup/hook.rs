use crate::{
    config::prelude::{Hook, HookContext, HookContextMetadata, HookExecType, Hooks},
    define_hook, log_info,
};
use inquire_derive::Selectable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Copy, Selectable, PartialEq)]
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
    local_path: Option<String>,
    remote_path: Option<String>,
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
        for backup_type in &self.types {
            match (backup_type, &self.exec) {
                (BackupType::Local, HookExecType::Push) => {
                    log_info!(
                        "executing backup {} in {}",
                        BackupType::Local,
                        HookExecType::Push
                    );

                    self.backup_local(&ctx)?;
                }
                (BackupType::Local, HookExecType::Pull) => {
                    log_info!(
                        "executing backup {} in {}",
                        BackupType::Local,
                        HookExecType::Pull
                    );

                    if let Some(local_path) =
                        ctx.metadata.get(&HookContextMetadata::SourceLocalPath)
                    {
                        let local_path = std::path::PathBuf::from(local_path);

                        if !local_path.exists() {
                            log_info!(
                                "local path does not exists, skipping backup: {:?}",
                                local_path
                            );

                            continue;
                        }

                        let local_ctx =
                            HookContext::new(local_path, &ctx.rclone_path, &ctx.remote_config);

                        self.backup_local(&local_ctx)?;
                    }
                }
                (BackupType::Remote, HookExecType::Push) => todo!(),
                (BackupType::Remote, HookExecType::Pull) => todo!(),
            }
        }

        Ok(ctx)
    }
}
