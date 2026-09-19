//! Mirror of `src/hooks/hook_builder.rs`.
//!
//! Only the non-interactive paths are covered here: building a `HookConfig`
//! or a shared `(HookConfig, HookConfig)` pair otherwise prompts the user.
//! The shared-config conversion, however, rejects hooks that do not support
//! it *before* prompting, which we can assert.

use rcloud::hooks::prelude::HookBuilder;
use rcloud::{HookConfig, Hooks};

#[test]
fn shared_config_unsupported_hook_errors_without_prompting() {
    // Zip does not support shared config; the pair conversion must fail fast
    // (before any prompt) with "operation not supported".
    let builder = HookBuilder::new(Hooks::Zip, None);
    let result: anyhow::Result<(HookConfig, HookConfig)> = builder.try_into();
    assert!(result.is_err());
}

#[test]
fn shared_config_unsupported_backup_errors_without_prompting() {
    let builder = HookBuilder::new(Hooks::Backup, None);
    let result: anyhow::Result<(HookConfig, HookConfig)> = builder.try_into();
    assert!(result.is_err());
}
