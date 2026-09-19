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
    /// The validated password, obtained (and verified) before transforming.
    type Acquired = String;

    /// Impure: prompts for the password (via the injected provider) and
    /// verifies it against the stored Argon2 hash.
    fn acquire(&self, ctx: &HookContext) -> anyhow::Result<String> {
        let password = ctx.dependencies.password.get()?;
        self.verify(&password)?;
        Ok(password)
    }

    /// Pure (given `password`): encrypts or decrypts the context path. Does no
    /// prompting; the password arrives as data.
    fn transform(
        &self,
        ctx: HookContext,
        cfg: &AppConfig,
        password: String,
    ) -> anyhow::Result<HookContext> {
        anyhow::ensure!(
            ctx.file_exists(),
            "source file does not exists: {:?}",
            ctx.path
        );

        let path = self
            .process_path(&ctx, cfg, &password)
            .with_context(|| format!("failed to process path: {}", ctx.path.display()))?;

        Ok(ctx.with_path(path))
    }
}
