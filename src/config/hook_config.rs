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

/// A single transformation applied to a path during a sync operation.
///
/// Hooks form an ordered pipeline. On push they run in declaration order
/// before the content is uploaded; on pull they run in reverse order after
/// the content is downloaded. Each hook receives the current [`HookContext`],
/// transforms the referenced path (compress, encrypt, back up, ...), and
/// returns an updated context whose `path` points at the transformed output.
///
/// Implementors must be `Send + Sync` because a context may be shared across
/// threads, and `Debug` so pipelines can be logged.
///
/// # Errors
/// Returns an error if the transformation fails (missing source, I/O error,
/// invalid configuration, ...). A failing hook aborts the whole sync.
pub trait Hook: std::fmt::Debug + Send + Sync {
    /// Transforms `ctx` and returns the resulting context.
    ///
    /// `cfg` provides application-wide settings such as the temp directory.
    fn process(&self, ctx: HookContext, cfg: &AppConfig) -> anyhow::Result<HookContext>;
}

/// The set of hook kinds known to the application.
///
/// This enum is the runtime discriminator used by the CLI/TUI to select which
/// hook to configure. The associated config/hook types and metadata (filename
/// impact, descriptions, shared-config support) are wired up by the
/// [`crate::register_hooks!`] macro.
#[derive(Debug, Clone, Copy, Selectable, PartialEq)]
pub enum Hooks {
    /// Compress with Zstd into a single archive.
    Zip,
    /// Create rotated backup replicas on local and/or remote storage.
    Backup,
    /// Encrypt/decrypt with AES-256-GCM using an Argon2-derived key.
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

/// Direction a hook runs in, and by extension the direction of a sync.
///
/// `Push` transforms local content on its way up to the remote; `Pull`
/// reverses the transformation on content coming back down.
#[derive(Debug, Clone, Serialize, Deserialize, Copy, Selectable, PartialEq, ValueEnum)]
pub enum HookExecType {
    /// Local -> remote (upload).
    Push,
    /// Remote -> local (download).
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
        modifies_name: true,
        share_config: true,
        display: |_cfg: &EncryptionHookConfig, f: &mut std::fmt::Formatter| write!(f, "Encryption"),
        push_desc: "Encrypt Path",
        pull_desc: "Decrypt Path",
    }
}
