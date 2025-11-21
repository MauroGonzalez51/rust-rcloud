use crate::cli::context::CommandContext;
use anyhow::Context;

pub fn registry_edit(context: CommandContext) -> anyhow::Result<()> {
    if let Some(config_path) = &context.global.registry {
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
