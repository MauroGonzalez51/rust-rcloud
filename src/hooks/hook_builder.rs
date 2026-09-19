use crate::{
    cli::prompter::Prompter,
    config::prelude::{HookConfig, HookExecType, Hooks},
    hooks::{
        backup::hook::BackupHookConfig, encryption::hook::EncryptionHookConfig,
        zip::hook::ZipHookConfig,
    },
};

/// Interactive constructor for a hook's config for a single direction.
///
/// Implemented by each `XHookConfig`. Prompts the user (compression level,
/// password, ...) through the injected [`Prompter`] and returns the
/// corresponding [`HookConfig`].
pub trait HookBuilderTrait: std::fmt::Debug + Send + Sync {
    /// Builds the config for the given `exec` direction.
    fn build<P: Prompter>(exec: HookExecType, prompter: &P) -> anyhow::Result<HookConfig>;
}

/// Interactive constructor that produces both push and pull configs at once.
///
/// Implemented by hooks whose two directions share setup input — e.g.
/// Encryption, where one password derives both the encrypt and decrypt config.
/// Gated by [`Hooks::share_config`](crate::config::prelude::Hooks).
pub trait HookBuilderSharedConfigTrait: std::fmt::Debug + Send + Sync {
    /// Builds the `(push, pull)` config pair from a single prompt session.
    fn build_shared<P: Prompter>(prompter: &P) -> anyhow::Result<(HookConfig, HookConfig)>;
}

/// Dispatches hook construction to the right `XHookConfig` builder.
#[derive(Debug)]
pub struct HookBuilder {
    hook_type: Hooks,
    hook_exec_type: Option<HookExecType>,
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

    /// Builds a single-direction config using `prompter`.
    pub fn build<P: Prompter>(&self, prompter: &P) -> anyhow::Result<HookConfig> {
        let exec = self
            .hook_exec_type
            .expect("exec_type should be declared");

        match self.hook_type {
            Hooks::Zip => ZipHookConfig::build(exec, prompter),
            Hooks::Backup => BackupHookConfig::build(exec, prompter),
            Hooks::Encryption => EncryptionHookConfig::build(exec, prompter),
        }
    }

    /// Builds a shared `(push, pull)` config pair using `prompter`, if the hook
    /// supports it.
    pub fn build_shared<P: Prompter>(
        &self,
        prompter: &P,
    ) -> Option<anyhow::Result<(HookConfig, HookConfig)>> {
        match self.hook_type {
            Hooks::Encryption => Some(EncryptionHookConfig::build_shared(prompter)),
            _ => None,
        }
    }
}
