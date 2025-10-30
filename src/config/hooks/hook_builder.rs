use crate::{
    config::{hooks::zip::ZipHookConfig, prelude::*},
    log_info,
};
use anyhow::{Context, bail};
use inquire::Text;

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
    fn build_push(
        self,
        hook_type: &Hooks,
        list: &[HookConfig],
        local_path: &str,
    ) -> anyhow::Result<HookConfig> {
        log_info!("configuring {} for {}", hook_type, HookExecType::Push);

        match hook_type {
            Hooks::Zip => {
                let level = Text::new("Compression level (0-9):")
                    .with_default("6")
                    .prompt()
                    .context("failed to get compression level")?
                    .parse::<i64>()
                    .context("failed to parse compresion level")?;

                let exclude = Text::new("Exclude patterns: ")
                    .with_help_message("comma-separated, glob only, optional")
                    .prompt_skippable()
                    .context("failed to get exclude patterns")?;

                let exclude = exclude.map(|s| {
                    s.split(',')
                        .map(|p| p.trim().to_string())
                        .filter(|p| !p.is_empty())
                        .collect()
                });

                Ok(HookConfig::Zip(ZipHookConfig {
                    exec: HookExecType::Push,
                    source: self.get_next_source_push(list, local_path),
                    level: Some(level),
                    exclude,
                }))
            }
        }
    }

    fn build_pull(
        self,
        hook_type: &Hooks,
        list: &[HookConfig],
        remote_path: &str,
    ) -> anyhow::Result<HookConfig> {
        log_info!("configuring {} for {}", hook_type, HookExecType::Pull);

        match hook_type {
            Hooks::Zip => Ok(HookConfig::Zip(ZipHookConfig {
                exec: HookExecType::Pull,
                source: self.get_next_source_pull(list, remote_path),
                level: None,
                exclude: None,
            })),
        }
    }

    pub fn build(self) -> anyhow::Result<HookConfig> {
        let hook_type = self.hook_type.context("missing hook_type")?;
        let hook_exec_type = self.hook_exec_type.context("missing hook_exec_type")?;
        let remote_path = self.remote_path.clone().context("missing remote_path")?;
        let local_path = self.local_path.clone().context("missing local_path")?;
        let list = self.list.clone().context("missing list")?;

        if hook_exec_type == HookExecType::Push {
            return self
                .build_push(&hook_type, &list, &local_path)
                .context("failed to build push hook");
        }

        if hook_exec_type == HookExecType::Pull {
            return self
                .build_pull(&hook_type, &list, &remote_path)
                .context("failed to build pull hook");
        }

        bail!("HookExecType::Both is not supported");
    }

    fn compute_hook_output(self, source: &str, hook_type: Hooks) -> String {
        match hook_type {
            Hooks::Zip => format!("{}.zip", source),
        }
    }

    fn get_next_source_push(self, hooks: &[HookConfig], local_path: &str) -> String {
        if hooks.is_empty() {
            return local_path.to_string();
        }
        let last_hook = &hooks[hooks.len() - 1];
        self.compute_hook_output(last_hook.source(), *last_hook.hook_type())
    }

    fn get_next_source_pull(self, hooks: &[HookConfig], remote_path: &str) -> String {
        if hooks.is_empty() {
            return remote_path.to_string();
        }
        let first_hook = &hooks[0];
        self.compute_hook_output(first_hook.source(), *first_hook.hook_type())
    }
}
