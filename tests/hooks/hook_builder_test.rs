//! Mirror of `src/hooks/hook_builder.rs`.
//!
//! Only the non-interactive paths are covered here: building a `HookConfig`
//! or a shared `(HookConfig, HookConfig)` pair otherwise prompts the user.
//! The shared-config path, however, rejects hooks that do not support it
//! *before* prompting, which we assert with a prompter that panics if used.

use crate::support::NoPrompter;
use rcloud::hooks::prelude::HookBuilder;
use rcloud::Hooks;

#[test]
fn shared_config_unsupported_hook_returns_none_without_prompting() {
    // Zip does not support shared config; build_shared must return None
    // (unsupported) without invoking the prompter.
    let builder = HookBuilder::new(Hooks::Zip, None);
    let result = builder.build_shared(&NoPrompter);
    assert!(result.is_none());
}

#[test]
fn shared_config_unsupported_backup_returns_none_without_prompting() {
    let builder = HookBuilder::new(Hooks::Backup, None);
    let result = builder.build_shared(&NoPrompter);
    assert!(result.is_none());
}
