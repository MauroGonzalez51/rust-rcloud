use crate::{
    config::prelude::AppConfig,
    hooks::prelude::{
        BackupHook, BackupHookConfig, EncryptionHook, EncryptionHookConfig, HookContext, ZipHook,
        ZipHookConfig,
    },
    register_hooks,
};
use clap::ValueEnum;
use inquire_derive::Selectable;
use serde::{Deserialize, Serialize};

pub trait Hook: std::fmt::Debug + Send + Sync {
    fn process(&self, ctx: HookContext, cfg: &AppConfig) -> anyhow::Result<HookContext>;
}

#[derive(Debug, Clone, Copy, Selectable, PartialEq)]
pub enum Hooks {
    Zip,
    Backup,
    Encryption,
}

impl std::fmt::Display for Hooks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Hooks::Zip => write!(f, "Zip"),
            Hooks::Backup => write!(f, "Backup"),
            Hooks::Encryption => write!(f, "Encryption"),
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
        share_config: false,
        display: |cfg: &ZipHookConfig, f: &mut std::fmt::Formatter| write!(f, "Zip(level: {:?})", cfg.level),
        push_desc: "Compress the file/folder before uploading",
        pull_desc: "Extract the file/folder after downloading",
    },
    Backup {
        config: BackupHookConfig,
        hook: BackupHook,
        enum_type: Hooks::Backup,
        modifies_name: false,
        share_config: false,
        display: |cfg: &BackupHookConfig, f: &mut std::fmt::Formatter| write!(f, "Backup(replicas: {})", cfg.replicas),
        push_desc: "Create a backup copy on Local/Remote",
        pull_desc: "Create a backup copy on Local/Remote",
    },
    Encryption {
        config: EncryptionHookConfig,
        hook: EncryptionHook,
        enum_type: Hooks::Encryption,
        modifies_name: false,
        share_config: true,
        display: |_cfg: &EncryptionHookConfig, f: &mut std::fmt::Formatter| write!(f, "Encryption"),
        push_desc: "Encrypt Path",
        pull_desc: "Decrypt Path",
    }
}
