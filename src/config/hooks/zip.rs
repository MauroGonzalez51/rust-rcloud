use crate::{
    config::hook_config::{Hook, HookContext, HookType},
    define_hook,
};
use std::path::PathBuf;

define_hook!(ZipHook {
    source: String,
    destination: String,
    level: Option<u8>,
    exclude: Option<Vec<String>>,
});

impl Hook for ZipHook {
    fn name(&self) -> &'static str {
        "zip"
    }

    fn exec_type(&self) -> &HookType {
        &self.exec
    }

    fn process(&self, ctx: HookContext) -> anyhow::Result<HookContext> {
        Ok(HookContext {
            file_path: PathBuf::from(self.destination.clone()),
            content: None,
            metadata: std::collections::HashMap::new(),
        })
    }
}
