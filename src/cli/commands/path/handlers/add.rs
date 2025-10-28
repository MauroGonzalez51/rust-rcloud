use crate::{
    cli::{commands::path::utils::path, parser::Args},
    config::prelude::*,
};
use anyhow::Context;

pub fn path_add(
    _args: &Args,
    registry: &Registry,
    remote_id: &Option<String>,
    local_path: &Option<String>,
    remote_path: &Option<String>,
) -> anyhow::Result<()> {
    let remote_id = match remote_id {
        Some(value) => value,
        None => &path::Prompt::remote_id::<
            fn(inquire::Select<'_, String>) -> inquire::Select<'_, String>,
        >(registry, None)
        .context("failed to get remote_id")?,
    };

    let local_path = match local_path {
        Some(value) => value,
        None => &path::Prompt::path("local path:")
            .prompt()
            .context("failed to get local path")?,
    };

    let remote_path = match remote_path {
        Some(value) => value,
        None => &path::Prompt::path("remote path")
            .prompt()
            .context("failed to get remote path")?,
    };

    let hooks = Vec::<Box<dyn Hook>>::new();

    let hook_type = path::Prompt::hook_type()
        .prompt()
        .context("failed to select hook type")?;

    Ok(())
}
