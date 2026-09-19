use crate::cli::prompter::Prompter;
use crate::config::prelude::{Registry, TagOption};
use anyhow::Context;

pub fn declare_tags<P: Prompter>(
    prompter: &P,
    registry: std::sync::Arc<std::sync::Mutex<Registry>>,
) -> anyhow::Result<Vec<String>> {
    let add_tags = prompter
        .confirm("Add some tags?", false)
        .context("failed to get confirmation")?;

    let mut tags: Vec<String> = vec![];

    if add_tags {
        tags = TagOption::multiple_select(
            prompter,
            "Select tags:",
            std::sync::Arc::clone(&registry),
            true,
            false,
        )
        .context("failed to select tags")?;
    }

    Ok(tags)
}

pub fn select_tags<P: Prompter>(
    prompter: &P,
    registry: std::sync::Arc<std::sync::Mutex<Registry>>,
) -> anyhow::Result<Vec<String>> {
    TagOption::multiple_select(
        prompter,
        "Select tags:",
        std::sync::Arc::clone(&registry),
        false,
        true,
    )
    .context("failed to select tags")
}
