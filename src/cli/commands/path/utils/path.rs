use crate::{cli::commands::remote::utils::remote, config::prelude::*};
use anyhow::Context;
use inquire::{Select, Text};

pub struct Prompt;

impl Prompt {
    pub fn remote_id<F>(
        registry: std::sync::Arc<std::sync::Mutex<Registry>>,
        f: Option<F>,
    ) -> anyhow::Result<String>
    where
        F: FnOnce(Select<'_, String>) -> Select<'_, String>,
    {
        let remote = remote::Prompt::remote("Select a remote:", registry, f)
            .context("failed to select remote")?;

        Ok(remote.id.clone())
    }

    pub fn path(message: &'static str) -> Text<'static, 'static> {
        Text::new(message).with_validator(inquire::validator::MinLengthValidator::new(1))
    }

    pub fn path_config(
        prompt: &str,
        registry: std::sync::Arc<std::sync::Mutex<Registry>>,
    ) -> anyhow::Result<String> {
        let options: Vec<(String, String)> = registry
            .lock()
            .map_err(|e| anyhow::anyhow!("{}", e))?
            .paths
            .iter()
            .map(|p| {
                (
                    format!("{} -> {}", p.local_path, p.remote_path),
                    p.id.clone(),
                )
            })
            .collect();

        let display_options = options.iter().map(|(display, _)| display.clone()).collect();

        let selected = Select::new(prompt, display_options)
            .with_vim_mode(true)
            .with_page_size(10)
            .with_help_message("<local_path> -> <remote_path>")
            .prompt()
            .context("failed to create select prompt")?;

        let selected_id = &options
            .into_iter()
            .find(|(display, _)| *display == selected)
            .map(|(_, id)| id)
            .ok_or_else(|| anyhow::anyhow!("failed to find selected path"))?;

        Ok(selected_id.clone())
    }
}
