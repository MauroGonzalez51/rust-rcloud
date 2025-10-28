use crate::{
    cli::{commands::path::utils::path, parser::Args},
    config::prelude::*,
    log_info, log_success, log_warn,
};
use anyhow::Context;

pub fn path_remove(
    _args: &Args,
    registry: &mut Registry,
    path_id: &Option<String>,
) -> anyhow::Result<()> {
    if registry.paths.is_empty() {
        log_warn!("no paths configured");
        return Ok(());
    }

    let path_id = match path_id {
        Some(value) => value,
        None => &path::Prompt::path_config("Select a record:", registry)
            .context("failed to select path config")?,
    };

    let path = registry
        .paths
        .iter()
        .find(|p| p.id == *path_id)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("path not found"))?;

    log_info!("Removing path: {} -> {}", path.local_path, path.remote_path);

    registry
        .tx(|rgx| {
            rgx.paths.retain(|r| r.id != path.id);
        })
        .context("failed to execute transaction")?;

    log_success!("path removed successfully");

    Ok(())
}
