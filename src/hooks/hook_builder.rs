use crate::{
    config::prelude::*,
    hooks::{backup::hook::BackupHookConfig, zip::hook::ZipHookConfig},
    log_debug,
};
use anyhow::Context;

pub struct NeedsHookType;
pub struct NeedsExecType;
pub struct Ready;

pub struct HookBuilder<State = NeedsHookType> {
    hook_type: Option<Hooks>,
    hook_exec_type: Option<HookExecType>,
    _state: std::marker::PhantomData<State>,
}

impl Default for HookBuilder {
    fn default() -> Self {
        Self {
            hook_type: None,
            hook_exec_type: None,
            _state: std::marker::PhantomData,
        }
    }
}

impl HookBuilder<NeedsHookType> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_hook_type(self, hook_type: Hooks) -> HookBuilder<NeedsExecType> {
        HookBuilder {
            hook_type: Some(hook_type),
            hook_exec_type: self.hook_exec_type,
            _state: std::marker::PhantomData,
        }
    }
}

impl HookBuilder<NeedsExecType> {
    pub fn with_exec_type(self, exec_type: HookExecType) -> HookBuilder<Ready> {
        HookBuilder {
            hook_type: self.hook_type,
            hook_exec_type: Some(exec_type),
            _state: std::marker::PhantomData,
        }
    }
}

impl HookBuilder<Ready> {
    pub fn build(self) -> anyhow::Result<HookConfig> {
        let hook_type = self.hook_type.context("missing hook_type")?;
        let hook_exec_type = self.hook_exec_type.context("missing hook_exec_type")?;

        let hook_config = match hook_type {
            Hooks::Zip => {
                ZipHookConfig::build(hook_exec_type).context("failed to build zip hook")?
            }
            Hooks::Backup => {
                BackupHookConfig::build(hook_exec_type).context("failed to build backup hook")?
            }
        };

        log_debug!("hook config: {}", hook_config);

        Ok(hook_config)
    }
}
