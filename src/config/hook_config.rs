use crate::config::hooks::zip::{ZipHook, ZipHookConfig};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookType {
    Push,
    Pull,
    Both,
}
pub trait Hook: std::fmt::Debug + Send + Sync {
    fn process(&self, ctx: HookContext) -> anyhow::Result<HookContext>;
    fn name(&self) -> &'static str;
    fn exec_type(&self) -> &HookType;
}

#[derive(Debug, Clone)]
pub struct HookContext {
    pub file_path: PathBuf,
    pub content: Option<Vec<u8>>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl HookContext {
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            content: None,
            metadata: std::collections::HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HookConfig {
    Zip(ZipHookConfig),
}

impl Into<Box<dyn Hook>> for HookConfig {
    fn into(self) -> Box<dyn Hook> {
        match self {
            HookConfig::Zip(cfg) => Box::new(ZipHook::from(cfg)),
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
                pub exec: HookType,
                $(pub $field: $field_ty),*
            }
        }

        #[derive(Debug)]
        pub struct $hook_name {
            pub exec: HookType,
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
