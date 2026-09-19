use crate::cli::prompter::{Prompter, TextOptions};
use crate::config::prelude::*;
use anyhow::Context;

pub struct Prompt;
pub struct Utils;

/// A remote wrapped for selection: shown as a rich label, selected by value.
struct RemoteChoice(Remote);

impl std::fmt::Display for RemoteChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({}) [{}]",
            self.0.remote_name,
            self.0.provider,
            &self.0.id[..8.min(self.0.id.len())]
        )
    }
}

impl Prompt {
    /// Prompts for a remote name.
    pub fn name<P: Prompter>(prompter: &P) -> anyhow::Result<String> {
        prompter.text(
            "Provide the remote name:",
            TextOptions::new().required().help(
                "Must be the same that you inserted when configuring the remote in 'rcloud'",
            ),
        )
    }

    /// Prompts for a remote name, prefilled with `default`.
    pub fn name_with_default<P: Prompter>(prompter: &P, default: &str) -> anyhow::Result<String> {
        prompter.text(
            "Provide the remote name:",
            TextOptions::new().required().default_value(default),
        )
    }

    /// Prompts for a remote provider.
    pub fn provider<P: Prompter>(prompter: &P) -> anyhow::Result<String> {
        prompter.text(
            "Provide the remote provider:",
            TextOptions::new().required(),
        )
    }

    /// Prompts for a remote provider, prefilled with `default`.
    pub fn provider_with_default<P: Prompter>(
        prompter: &P,
        default: &str,
    ) -> anyhow::Result<String> {
        prompter.text(
            "Provide the remote provider:",
            TextOptions::new().required().default_value(default),
        )
    }

    /// Selects a configured remote by presenting a labelled list.
    pub fn remote<P: Prompter>(
        prompter: &P,
        message: &str,
        registry: std::sync::Arc<std::sync::Mutex<Registry>>,
    ) -> anyhow::Result<Remote> {
        let choices: Vec<RemoteChoice> = registry
            .lock()
            .map_err(|e| anyhow::anyhow!("{}", e))?
            .remotes
            .iter()
            .cloned()
            .map(RemoteChoice)
            .collect();

        anyhow::ensure!(!choices.is_empty(), "no remotes configured");

        let chosen = prompter
            .select(message, choices)
            .context("failed to select remote")?;

        Ok(chosen.0)
    }
}

impl Utils {
    pub fn remote_by_id(
        registry: std::sync::Arc<std::sync::Mutex<Registry>>,
        id: &String,
    ) -> anyhow::Result<Remote> {
        registry
            .lock()
            .map_err(|e| anyhow::anyhow!("{}", e))?
            .remotes
            .iter()
            .find(|r| r.id == *id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("remote not found"))
    }
}
