use crate::config::{hooks::zip::ZipHookConfig, prelude::*};
use anyhow::Context;

pub struct NeedsHookType;
pub struct NeedsExecType;
pub struct NeedsPaths;
pub struct NeedsList;
pub struct Ready;

pub struct HookBuilder<State = NeedsHookType> {
    hook_type: Option<Hooks>,
    hook_exec_type: Option<HookExecType>,
    remote_path: Option<String>,
    local_path: Option<String>,
    list: Option<Vec<HookConfig>>,
    _state: std::marker::PhantomData<State>,
}

impl Default for HookBuilder {
    fn default() -> Self {
        Self {
            hook_type: None,
            hook_exec_type: None,
            remote_path: None,
            local_path: None,
            list: None,
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
            remote_path: self.remote_path,
            local_path: self.local_path,
            list: self.list,
            _state: std::marker::PhantomData,
        }
    }
}

impl HookBuilder<NeedsExecType> {
    pub fn with_exec_type(self, exec_type: HookExecType) -> HookBuilder<NeedsPaths> {
        HookBuilder {
            hook_type: self.hook_type,
            hook_exec_type: Some(exec_type),
            remote_path: self.remote_path,
            local_path: self.local_path,
            list: self.list,
            _state: std::marker::PhantomData,
        }
    }
}

impl HookBuilder<NeedsPaths> {
    pub fn with_paths(self, local: String, remote: String) -> HookBuilder<NeedsList> {
        HookBuilder {
            hook_type: self.hook_type,
            hook_exec_type: self.hook_exec_type,
            remote_path: Some(remote),
            local_path: Some(local),
            list: self.list,
            _state: std::marker::PhantomData,
        }
    }
}

impl HookBuilder<NeedsList> {
    pub fn with_list(self, list: &[HookConfig]) -> HookBuilder<Ready> {
        HookBuilder {
            hook_type: self.hook_type,
            hook_exec_type: self.hook_exec_type,
            remote_path: self.remote_path,
            local_path: self.local_path,
            list: Some(list.to_vec()),
            _state: std::marker::PhantomData,
        }
    }
}

impl HookBuilder<Ready> {
    pub fn build(self) -> anyhow::Result<HookConfig> {
        let hook_type = self.hook_type.context("missing hook_type")?;
        let hook_exec_type = self.hook_exec_type.context("missing hook_exec_type")?;
        let remote_path = self.remote_path.clone().context("missing remote_path")?;
        let local_path = self.local_path.clone().context("missing local_path")?;
        let list = self.list.clone().context("missing list")?;

        let hook_config = match hook_type {
            Hooks::Zip => match hook_exec_type {
                HookExecType::Push => ZipHookConfig::build(
                    hook_exec_type,
                    &self.get_next_source_push(&list, &local_path),
                )
                .context("failed to build hook")?,

                HookExecType::Pull => ZipHookConfig::build(
                    hook_exec_type,
                    &self.get_next_source_pull(&list, &remote_path),
                )
                .context("failed to build hook")?,
            },
            Hooks::Backup => match hook_exec_type {
                HookExecType::Push => todo!(),
                HookExecType::Pull => todo!(),
            },
        };

        Ok(hook_config)
    }

    fn compute_hook_output(&self, hook_config: &HookConfig) -> String {
        match hook_config {
            HookConfig::Zip(cfg) => {
                let base_name = std::path::Path::new(&cfg.source)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("output");

                format!("{}.zip", base_name)
            }
            HookConfig::Backup(_cfg) => {
                todo!()
            }
        }
    }

    fn get_next_source_push(&self, hooks: &[HookConfig], local_path: &str) -> String {
        if hooks.is_empty() {
            return local_path.to_string();
        }

        let last_hook = &hooks[hooks.len() - 1];
        self.compute_hook_output(last_hook)
    }

    fn get_next_source_pull(self, hooks: &[HookConfig], remote_path: &str) -> String {
        if hooks.is_empty() {
            return remote_path.to_string();
        }

        let first_hook = &hooks[0];

        match first_hook {
            HookConfig::Zip(_) => {
                let base_name = std::path::Path::new(remote_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("archive");

                format!("{}.zip", base_name)
            }
            HookConfig::Backup(_) => todo!(),
        }
    }
}
