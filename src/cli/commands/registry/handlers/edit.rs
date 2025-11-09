use anyhow::Context;

use crate::cli::parser::Args;

pub fn registry_edit(args: &Args) -> anyhow::Result<()> {
    if let Some(config_path) = &args.registry {
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());

        std::process::Command::new(editor)
            .arg(
                config_path
                    .to_str()
                    .context("failed to convert path to string")?,
            )
            .spawn()
            .context("failed to launch editor")?;
    }

    Ok(())
}
