use crate::{
    config::prelude::{AppConfig, Hook},
    define_hook,
    hooks::prelude::HookContext,
};
use anyhow::Context;

define_hook!(EncryptionHook {
    hash: String,
});

impl Hook for EncryptionHook {
    fn process(&self, ctx: HookContext, cfg: &AppConfig) -> anyhow::Result<HookContext> {
        anyhow::ensure!(
            ctx.file_exists(),
            "source file does not exists: {:?}",
            &ctx.path
        );

        let derived_key = self.derive_key()?;

        let path = self
            .process_path(&ctx, cfg, &derived_key)
            .with_context(|| format!("failed to process path: {}", &ctx.path.display()))?;

        Ok(ctx.with_path(path))
    }
}
