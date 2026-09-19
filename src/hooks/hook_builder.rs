use crate::{
    config::prelude::{HookConfig, HookExecType, Hooks},
    hooks::{
        backup::hook::BackupHookConfig, encryption::hook::EncryptionHookConfig,
        zip::hook::ZipHookConfig,
    },
};

/// Interactive constructor for a hook's config for a single direction.
///
/// Implemented by each `XHookConfig`. Typically prompts the user (compression
/// level, password, ...) and returns the corresponding [`HookConfig`].
pub trait HookBuilderTrait: std::fmt::Debug + Send + Sync {
    /// Builds the config for the given `exec` direction.
    fn build(exec: HookExecType) -> anyhow::Result<HookConfig>;
}

/// Interactive constructor that produces both push and pull configs at once.
///
/// Implemented by hooks whose two directions share setup input — e.g.
/// Encryption, where one password derives both the encrypt and decrypt config.
/// Gated by [`Hooks::share_config`](crate::config::prelude::Hooks).
pub trait HookBuilderSharedConfigTrait: std::fmt::Debug + Send + Sync {
    /// Builds the `(push, pull)` config pair from a single prompt session.
    fn build_shared() -> anyhow::Result<(HookConfig, HookConfig)>;
}

/// Dispatches hook construction to the right `XHookConfig` builder.
///
/// Convert into a [`HookConfig`] (single direction) or a
/// `(HookConfig, HookConfig)` pair (shared config) via `TryFrom`/`TryInto`.
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
    /// Creates a builder for `hook_type`. Pass `hook_exec_type` for a
    /// single-direction build, or `None` when building a shared config pair.
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
