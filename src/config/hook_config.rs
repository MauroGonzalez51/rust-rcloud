use crate::{
    config::prelude::AppConfig,
    hooks::prelude::{BackupHook, BackupHookConfig, HookContext, ZipHook, ZipHookConfig},
    register_hooks,
};
use clap::ValueEnum;
use inquire_derive::Selectable;
use serde::{Deserialize, Serialize};

pub trait Hook: std::fmt::Debug + Send + Sync {
    fn process(&self, ctx: HookContext, cfg: &AppConfig) -> anyhow::Result<HookContext>;
}

#[derive(Debug, Clone, Copy, Selectable)]
pub enum Hooks {
    Zip,
    Backup,
}

impl std::fmt::Display for Hooks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Hooks::Zip => write!(f, "Zip Compression"),
            Hooks::Backup => write!(f, "Backup"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, Selectable, PartialEq, ValueEnum)]
pub enum HookExecType {
    Push,
    Pull,
}

impl std::fmt::Display for HookExecType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookExecType::Push => write!(f, "Push"),
            HookExecType::Pull => write!(f, "Pull"),
        }
    }
}

register_hooks! {
    Zip {
        config: ZipHookConfig,
        hook: ZipHook,
        enum_type: Hooks::Zip,
        modifies_name: true,
        display: |cfg: &ZipHookConfig, f: &mut std::fmt::Formatter| write!(f, "Zip(level: {:?})", cfg.level)
    },
    Backup {
        config: BackupHookConfig,
        hook: BackupHook,
        enum_type: Hooks::Backup,
        modifies_name: false,
        display: |cfg: &BackupHookConfig, f: &mut std::fmt::Formatter| write!(f, "Backup(replicas: {})", cfg.replicas)
    }
}
