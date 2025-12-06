use crate::config::prelude::{Registry, TagOption};
use anyhow::Context;
use inquire::Confirm;

pub fn declare_tags(registry: &Registry) -> anyhow::Result<Vec<String>> {
    let add_tags = Confirm::new("Add some tags?")
        .with_default(false)
        .prompt()
        .context("failed to get confirmation")?;

    let mut tags: Vec<String> = vec![];

    if add_tags {
        tags = TagOption::multiple_select("Select tags:", registry)
            .context("failed to select tags")?;
    }

    Ok(tags)
}
