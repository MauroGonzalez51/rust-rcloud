use crate::config::prelude::*;
use anyhow::Context;
use inquire::{MultiSelect, Text};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagOption {
    Existing(String),
    AddNew,
}

impl std::fmt::Display for TagOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TagOption::Existing(tag) => write!(f, "{}", tag),
            TagOption::AddNew => write!(f, "( Add new tag ...)"),
        }
    }
}

impl TagOption {
    pub fn multiple_select(
        msg: &str,
        registry: &Registry,
        allow_create_new_tags: bool,
        allow_empty: bool,
    ) -> anyhow::Result<Vec<String>> {
        let existing_tags: Vec<String> = registry
            .paths
            .iter()
            .flat_map(|path| path.tags.clone())
            .collect();

        let existing_tags: Vec<String> = existing_tags
            .into_iter()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let mut existing_tags: Vec<TagOption> = existing_tags
            .iter()
            .map(|t| TagOption::Existing(t.clone()))
            .collect();

        if allow_create_new_tags {
            existing_tags.push(TagOption::AddNew);
        }

        let mut selected_tags = Vec::new();

        loop {
            let default_indices: Vec<usize> = existing_tags
                .iter()
                .enumerate()
                .filter_map(|(i, tag)| {
                    if let TagOption::Existing(t) = tag
                        && selected_tags.contains(t)
                    {
                        return Some(i);
                    }

                    None
                })
                .collect();

            let selections = MultiSelect::new(msg, existing_tags.clone())
                .with_vim_mode(true)
                .with_default(&default_indices)
                .prompt()
                .context("failed to select tags")?;

            if !allow_empty && selections.is_empty() {
                continue;
            }

            let mut should_continue = false;

            selected_tags.clear();

            for selection in &selections {
                if let TagOption::Existing(tag) = selection
                    && !selected_tags.contains(tag)
                {
                    selected_tags.push(tag.clone());
                }
            }

            if selections.contains(&TagOption::AddNew) {
                let new_tag = Text::new("New Tag:")
                    .prompt()
                    .context("failed to get new tag")?
                    .trim()
                    .to_string();

                if !new_tag.is_empty() && !selected_tags.contains(&new_tag) {
                    selected_tags.push(new_tag.clone());

                    existing_tags.insert(existing_tags.len() - 1, TagOption::Existing(new_tag));

                    should_continue = true;
                }
            }

            if !should_continue {
                return Ok(selected_tags);
            }
        }
    }
}
