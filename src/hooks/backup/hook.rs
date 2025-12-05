use crate::{
    config::prelude::{AppConfig, Hook, HookExecType},
    define_hook,
    hooks::prelude::HookContext,
    log_info,
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
    fn process(&self, ctx: HookContext, _cfg: &AppConfig) -> anyhow::Result<HookContext> {
        for backup_type in &self.types {
            log_info!("executing backup {} in {}", backup_type, self.exec);

            match (backup_type, &self.exec) {
                (BackupType::Local, HookExecType::Push) => {
                    self.backup_local(&ctx)?;
                }
                (BackupType::Local, HookExecType::Pull) => {
                    let local_path = std::path::PathBuf::from(&ctx.path_config.local_path);

                    if !local_path.exists() {
                        log_info!(
                            "local path does not exist, skipping backup: {:?}",
                            local_path
                        );

                        continue;
                    }

                    self.backup_local(&HookContext::new(
                        local_path,
                        &ctx.rclone_path,
                        &ctx.remote_config,
                        &ctx.path_config,
                    ))?;
                }
                (BackupType::Remote, HookExecType::Push) => {
                    self.backup_remote(&ctx)?;
                }
                (BackupType::Remote, HookExecType::Pull) => {
                    self.backup_remote(&ctx)?;
                }
            }
        }

        Ok(ctx)
    }
}
