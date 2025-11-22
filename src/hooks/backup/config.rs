use crate::{
    config::prelude::{HookConfig, HookExecType, Hooks},
    hooks::backup::{BackupHookConfig, BackupType},
    log_info,
};
use anyhow::Context;

impl BackupHookConfig {
    fn prompt_if(
        types: &[BackupType],
        variant: BackupType,
        prompt: &str,
    ) -> anyhow::Result<Option<String>> {
        if types.contains(&variant) {
            return Ok(Some(
                inquire::Text::new(prompt)
                    .prompt()
                    .with_context(|| format!("failed to get path for {}", variant))?,
            ));
        }

        Ok(None)
    }

    pub fn build(exec_type: HookExecType) -> anyhow::Result<HookConfig> {
        log_info!("configuring {} for {}", Hooks::Backup, exec_type);

        let types = BackupType::multi_select("Select backup type(s):")
            .prompt()
            .context("failed to select backup types")?;

        let local_path =
            BackupHookConfig::prompt_if(&types, BackupType::Local, "Local Backup path:")
                .context("failed to get local path")?;

        let remote_path =
            BackupHookConfig::prompt_if(&types, BackupType::Remote, "Remote Backup path:")
                .context("failed to get remote path")?;

        let replicas = inquire::Text::new("Max replicas:")
            .with_default("1")
            .prompt()
            .context("invalid replicas")?
            .parse::<u32>()
            .context("not a number")?;

        Ok(HookConfig::Backup(Self {
            exec: exec_type,
            types,
            local_path,
            remote_path,
            replicas,
        }))
    }
}
