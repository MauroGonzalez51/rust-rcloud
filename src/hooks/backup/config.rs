use crate::{
    cli::prompter::{Prompter, TextOptions},
    config::prelude::{HookConfig, HookExecType, Hooks},
    hooks::prelude::{BackupHookConfig, BackupType, HookBuilderTrait},
    log_info, utils,
};
use anyhow::Context;

impl HookBuilderTrait for BackupHookConfig {
    fn build<P: Prompter>(exec: HookExecType, prompter: &P) -> anyhow::Result<HookConfig> {
        log_info!("configuring {} for {}", Hooks::Backup, exec);

        let types = prompter
            .multi_select(
                "Select backup type(s):",
                vec![BackupType::Local, BackupType::Remote],
                &[],
            )
            .context("failed to select backup types")?;

        let local_path = Self::prompt_if(prompter, &types, BackupType::Local, "Local Backup path:")
            .context("failed to get local path")?;

        let local_path = match local_path {
            Some(value) => Some(utils::expand_path(&value)?.to_string_lossy().to_string()),
            None => None,
        };

        let remote_path =
            Self::prompt_if(prompter, &types, BackupType::Remote, "Remote Backup path:")
                .context("failed to get remote path")?;

        let replicas = prompter
            .text("Max replicas:", TextOptions::new().default_value("1"))
            .context("invalid replicas")?
            .parse::<u32>()
            .context("not a number")?;

        Ok(HookConfig::Backup(Self {
            exec,
            types,
            local_path,
            remote_path,
            replicas,
        }))
    }
}

impl BackupHookConfig {
    fn prompt_if<P: Prompter>(
        prompter: &P,
        types: &[BackupType],
        variant: BackupType,
        prompt: &str,
    ) -> anyhow::Result<Option<String>> {
        if types.contains(&variant) {
            return Ok(Some(
                prompter
                    .text(prompt, TextOptions::new().required())
                    .with_context(|| format!("failed to get path for {}", variant))?,
            ));
        }

        Ok(None)
    }
}
