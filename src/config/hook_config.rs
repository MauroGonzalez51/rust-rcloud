use crate::config::hooks::zip::{ZipHook, ZipHookConfig};
use inquire_derive::Selectable;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub trait Hook: std::fmt::Debug + Send + Sync {
    fn process(&self, ctx: HookContext) -> anyhow::Result<HookContext>;
    fn name(&self) -> &'static str;
    fn exec_type(&self) -> &HookExecType;
    fn hook_type(&self) -> &Hooks;
}

#[derive(Debug, Clone, Copy, Selectable)]
pub enum Hooks {
    Zip,
}

impl std::fmt::Display for Hooks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Hooks::Zip => write!(f, "Zip Compression"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, Selectable, PartialEq)]
pub enum HookExecType {
    Push,
    Pull,
}

impl std::fmt::Display for HookExecType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookExecType::Push => write!(f, "{:?}", "Push"),
            HookExecType::Pull => write!(f, "{:?}", "Pull"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HookContext {
    pub path: PathBuf,
    pub metadata: std::collections::HashMap<String, String>,
}

impl HookContext {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn file_exists(&self) -> bool {
        self.path.exists()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HookConfig {
    Zip(ZipHookConfig),
}

impl From<HookConfig> for Box<dyn Hook> {
    fn from(val: HookConfig) -> Self {
        match val {
            HookConfig::Zip(cfg) => Box::new(ZipHook::from(cfg)),
        }
    }
}

impl std::fmt::Display for HookConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookConfig::Zip(cfg) => {
                write!(f, "Zip(level: {:?}, source: {})", cfg.level, cfg.source)
            }
        }
    }
}

impl HookConfig {
    pub fn source(&self) -> &String {
        match self {
            HookConfig::Zip(cfg) => &cfg.source,
        }
    }

    pub fn exec_type(&self) -> &HookExecType {
        match self {
            HookConfig::Zip(cfg) => &cfg.exec,
        }
    }

    pub fn hook_type(&self) -> &Hooks {
        match self {
            HookConfig::Zip(_) => &Hooks::Zip,
        }
    }
}

#[macro_export]
macro_rules! define_hook {
    (
        $hook_name:ident {
            $($field:ident: $field_ty:ty),* $(,)?
        }
    ) => {
        paste::paste! {
            #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
            pub struct [<$hook_name Config>] {
                pub exec: $crate::config::hook_config::HookExecType,
                $(pub $field: $field_ty),*
            }
        }

        #[derive(Debug)]
        pub struct $hook_name {
            pub exec: $crate::config::hook_config::HookExecType,
            $(pub $field: $field_ty),*
        }

        paste::paste! {
            impl From<[<$hook_name Config>]> for $hook_name {
                fn from(config: [<$hook_name Config>]) -> Self {
                    Self {
                        exec: config.exec,
                        $($field: config.$field),*
                    }
                }
            }
        }
    };
}
