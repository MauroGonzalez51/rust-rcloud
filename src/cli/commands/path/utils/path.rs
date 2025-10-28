use crate::{cli::commands::remote::utils::remote, config::prelude::*};
use anyhow::Context;
use inquire::{Select, Text};

pub struct Prompt;

impl Prompt {
    pub fn remote_id<F>(registry: &Registry, f: Option<F>) -> anyhow::Result<String>
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
}
