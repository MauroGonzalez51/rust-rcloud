use crate::{
    config::prelude::{HookConfig, HookExecType, Hooks},
    hooks::{
        backup::hook::BackupHookConfig, encryption::hook::EncryptionHookConfig,
        zip::hook::ZipHookConfig,
    },
};

pub trait HookBuilderTrait: std::fmt::Debug + Send + Sync {
    fn build(exec: HookExecType) -> anyhow::Result<HookConfig>;
}

pub trait HookBuilderSharedConfigTrait: std::fmt::Debug + Send + Sync {
    fn build_shared() -> anyhow::Result<(HookConfig, HookConfig)>;
}

#[derive(Debug)]
pub struct HookBuilder {
    hook_type: Hooks,
    hook_exec_type: Option<HookExecType>,
}

impl TryFrom<HookBuilder> for HookConfig {
    type Error = anyhow::Error;

    fn try_from(builder: HookBuilder) -> anyhow::Result<Self> {
        builder.build(
            builder.hook_type,
            builder
                .hook_exec_type
                .expect("exec_type should be declared"),
        )
    }
}

impl TryFrom<HookBuilder> for (HookConfig, HookConfig) {
    type Error = anyhow::Error;

    fn try_from(builder: HookBuilder) -> anyhow::Result<Self> {
        builder
            .build_shared(builder.hook_type)
            .ok_or_else(|| anyhow::anyhow!("operation not supported"))?
    }
}

impl HookBuilder {
    pub fn new(hook_type: Hooks, hook_exec_type: Option<HookExecType>) -> Self {
        Self {
            hook_type,
            hook_exec_type,
        }
    }

    fn build(&self, hook_type: Hooks, exec: HookExecType) -> anyhow::Result<HookConfig> {
        match hook_type {
            Hooks::Zip => ZipHookConfig::build(exec),
            Hooks::Backup => BackupHookConfig::build(exec),
            Hooks::Encryption => EncryptionHookConfig::build(exec),
        }
    }

    fn build_shared(&self, hook_type: Hooks) -> Option<anyhow::Result<(HookConfig, HookConfig)>> {
        match hook_type {
            Hooks::Encryption => Some(EncryptionHookConfig::build_shared()),
            _ => None,
        }
    }
}
