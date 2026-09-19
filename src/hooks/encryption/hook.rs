use crate::{
    config::prelude::{AppConfig, Hook},
    define_hook,
    hooks::prelude::HookContext,
};
use anyhow::Context;

define_hook!(EncryptionHook { hash: String });

/// Encrypts (push) or decrypts (pull) the context path with AES-256-GCM.
///
/// The password is prompted and verified against the stored Argon2 hash, then
/// a per-file key is derived from a random salt stored in each file's header.
impl Hook for EncryptionHook {
    fn process(&self, ctx: HookContext, cfg: &AppConfig) -> anyhow::Result<HookContext> {
        anyhow::ensure!(
            ctx.file_exists(),
            "source file does not exists: {:?}",
            ctx.path
        );

        let password = self.verify_password()?;

        let path = self
            .process_path(&ctx, cfg, &password)
            .with_context(|| format!("failed to process path: {}", ctx.path.display()))?;

        Ok(ctx.with_path(path))
    }
}
