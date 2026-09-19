use crate::cli::{commands::path::utils::path, context::CommandContext};
use crate::hooks::prelude::{ProcessRcloneRunner, RcloneRunner};
use anyhow::Context;

fn list_and_print(rclone_path: &str, path: &str) -> anyhow::Result<()> {
    let runner = ProcessRcloneRunner::new(rclone_path);
    let entries = runner
        .list(path)
        .with_context(|| format!("failed to execute rclone ls {}", path))?;

    for entry in entries {
        println!("{}", entry);
    }

    Ok(())
}

#[derive(Clone)]
pub struct LocalArgs<'a> {
    pub path: &'a Option<String>,
    pub path_config: &'a Option<String>,
}

impl<'a> Default for LocalArgs<'a> {
    fn default() -> Self {
        Self {
            path: &None,
            path_config: &None,
        }
    }
}

pub fn remote_ls(context: CommandContext<LocalArgs>) -> anyhow::Result<()> {
    if let Some(path) = context.local.path {
        list_and_print(&context.global.rclone, path)?;
        return Ok(());
    }

    let path_id = match context.local.path_config {
        Some(id) => id,
        None => {
            &path::Prompt::path_config("Select the path:", std::sync::Arc::clone(&context.registry))
                .context("failed to select path")?
        }
    };

    let binding = context.with_registry()?;

    let path_config = binding
        .paths
        .iter()
        .find(|p| p.id == *path_id)
        .ok_or_else(|| anyhow::anyhow!("path does not exists"))?;

    let remote_config = binding
        .remotes
        .iter()
        .find(|r| r.id == *path_config.remote_id)
        .ok_or_else(|| anyhow::anyhow!("remote does not exists"))?;

    list_and_print(
        &context.global.rclone,
        &format!("{}:{}", remote_config.remote_name, path_config.remote_path),
    )?;

    Ok(())
}
