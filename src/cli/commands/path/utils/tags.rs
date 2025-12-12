use crate::config::prelude::{Registry, TagOption};
use anyhow::Context;
use inquire::Confirm;

pub fn declare_tags(
    registry: std::sync::Arc<std::sync::Mutex<Registry>>,
) -> anyhow::Result<Vec<String>> {
    let add_tags = Confirm::new("Add some tags?")
        .with_default(false)
        .prompt()
        .context("failed to get confirmation")?;

    let mut tags: Vec<String> = vec![];

    if add_tags {
        tags = TagOption::multiple_select(
            "Select tags:",
            std::sync::Arc::clone(&registry),
            true,
            false,
        )
        .context("failed to select tags")?;
    }

    Ok(tags)
}

pub fn select_tags(
    registry: std::sync::Arc<std::sync::Mutex<Registry>>,
) -> anyhow::Result<Vec<String>> {
    TagOption::multiple_select(
        "Select tags:",
        std::sync::Arc::clone(&registry),
        false,
        true,
    )
    .context("failed to select tags")
}
