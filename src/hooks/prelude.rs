pub use super::backup::{BackupHook, BackupHookConfig, BackupType};
// Re-exported as public API for downstream consumers and tests; not all names
// are referenced inside the crate itself.
#[allow(unused_imports)]
pub use super::dependencies::{
    HookDependencies, InteractivePasswordProvider, PasswordProvider, ProcessRcloneRunner,
    RcloneRunner,
};
pub use super::encryption::{EncryptionHook, EncryptionHookConfig};
pub use super::hook_builder::{HookBuilder, HookBuilderSharedConfigTrait, HookBuilderTrait};
pub use super::hook_context::{HookContext, HookContextMetadata};
pub use super::zip::{ZipHook, ZipHookConfig};
