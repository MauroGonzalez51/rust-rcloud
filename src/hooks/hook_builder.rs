use crate::{
    config::prelude::*,
    hooks::{backup::hook::BackupHookConfig, zip::hook::ZipHookConfig},
};
use anyhow::{Context, Ok};
use bon::Builder;

#[derive(Builder)]
pub struct HookBuilder {
    #[builder(required)]
    hook_type: Option<Hooks>,

    #[builder(required)]
    hook_exec_type: Option<HookExecType>,
}

impl TryFrom<HookBuilder> for HookConfig {
    type Error = anyhow::Error;

    fn try_from(builder: HookBuilder) -> anyhow::Result<Self> {
        let hook_type = builder.hook_type.expect("hook type must be declared");

        let exec_type = builder
            .hook_exec_type
            .expect("hook exec type must be declared");

        let config = match hook_type {
            Hooks::Zip => ZipHookConfig::build(exec_type).context("failed to build zip hook")?,
            Hooks::Backup => {
                BackupHookConfig::build(exec_type).context("failed to build backup hook")?
            }
        };

        Ok(config)
    }
}
