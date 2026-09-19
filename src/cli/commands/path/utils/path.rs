use crate::{
    cli::{commands::remote::utils::remote, prompter::Prompter},
    config::prelude::*,
};
use anyhow::Context;
use crate::cli::prompter::TextOptions;

pub struct Prompt;

/// A path config wrapped for selection: shown as a label, selected by value.
struct PathChoice {
    id: String,
    label: String,
}

impl std::fmt::Display for PathChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}

impl Prompt {
    /// Selects a remote and returns its id.
    pub fn remote_id<P: Prompter>(
        prompter: &P,
        registry: std::sync::Arc<std::sync::Mutex<Registry>>,
    ) -> anyhow::Result<String> {
        let remote = remote::Prompt::remote(prompter, "Select a remote:", registry)
            .context("failed to select remote")?;

        Ok(remote.id)
    }

    /// Prompts for a filesystem path (non-empty).
    pub fn path<P: Prompter>(prompter: &P, message: &str) -> anyhow::Result<String> {
        prompter.text(message, TextOptions::new().required())
    }

    /// Selects a configured path and returns its id.
    pub fn path_config<P: Prompter>(
        prompter: &P,
        message: &str,
        registry: std::sync::Arc<std::sync::Mutex<Registry>>,
    ) -> anyhow::Result<String> {
        let choices: Vec<PathChoice> = registry
            .lock()
            .map_err(|e| anyhow::anyhow!("{}", e))?
            .paths
            .iter()
            .map(|p| PathChoice {
                id: p.id.clone(),
                label: format!("{} -> {}", p.local_path, p.remote_path),
            })
            .collect();

        anyhow::ensure!(!choices.is_empty(), "no paths configured");

        let chosen = prompter
            .select(message, choices)
            .context("failed to create select prompt")?;

        Ok(chosen.id)
    }
}
