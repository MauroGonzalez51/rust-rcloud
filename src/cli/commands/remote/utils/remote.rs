use crate::config::prelude::*;
use anyhow::Context;
use inquire::{Select, Text};

pub struct Prompt;
pub struct Utils;

impl Prompt {
    pub fn name() -> Text<'static, 'static> {
        Text::new("Provide the remote name:")
            .with_validator(inquire::validator::MinLengthValidator::new(1))
    }

    pub fn provider() -> Text<'static, 'static> {
        Text::new("Provide the remote provider:")
            .with_validator(inquire::validator::MinLengthValidator::new(1))
    }

    pub fn remote<F>(registry: &Registry, f: Option<F>) -> anyhow::Result<Remote>
    where
        F: FnOnce(Select<'_, String>) -> Select<'_, String>,
    {
        let options: Vec<(String, String)> = registry
            .remotes
            .iter()
            .map(|r| {
                (
                    format!("{} ({}) [{}]", r.remote_name, r.provider, &r.id[..8]),
                    r.id.clone(),
                )
            })
            .collect();

        let display_options = options.iter().map(|(display, _)| display.clone()).collect();

        let mut select = Select::new("Select remote to remove", display_options)
            .with_vim_mode(true)
            .with_page_size(10)
            .with_help_message("<remote_name> (<remote_provider>) [<...remote_id>]");

        if let Some(modifier) = f {
            select = modifier(select);
        }

        let selected = select.prompt().context("failed to create select prompt")?;

        let selected_id = &options
            .into_iter()
            .find(|(display, _)| *display == selected)
            .map(|(_, id)| id)
            .ok_or_else(|| anyhow::anyhow!("failed to find selected remote"))?;

        registry
            .remotes
            .iter()
            .find(|r| r.id == *selected_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("remote not found"))
    }
}

impl Utils {
    pub fn remote_by_id(registry: &Registry, id: &String) -> anyhow::Result<Remote> {
        registry
            .remotes
            .iter()
            .find(|r| r.id == *id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("remote not found"))
    }
}
