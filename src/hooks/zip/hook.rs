use crate::{
    config::prelude::{AppConfig, Hook},
    define_hook,
    hooks::prelude::HookContext,
};

define_hook!(ZipHook {
    level: Option<i64>,
    exclude: Option<Vec<String>>,
});

/// Compresses (push) or extracts (pull) the context path with Zstd.
///
/// On push the source is packed into a single archive and a
/// [`ZipChecksum`](crate::hooks::prelude::HookContextMetadata::ZipChecksum) is
/// recorded; on pull the archive is unpacked into a temp directory.
impl Hook for ZipHook {
    /// Zip needs nothing from the outside world.
    type Acquired = ();

    fn acquire(&self, _ctx: &HookContext) -> anyhow::Result<()> {
        Ok(())
    }

    fn transform(
        &self,
        ctx: HookContext,
        cfg: &AppConfig,
        _acquired: (),
    ) -> anyhow::Result<HookContext> {
        anyhow::ensure!(
            ctx.file_exists(),
            "source file does not exists: {:?}",
            ctx.path
        );

        self.process_path(&ctx, cfg)
    }
}
