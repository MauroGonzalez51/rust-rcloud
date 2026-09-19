use crate::{
    cli::{context::CommandContext, output::OutputSink, prompter::Prompter},
    config::prelude::Registry,
    log_info,
};
use anyhow::Context;

pub fn configure_setup<P: Prompter, S: OutputSink>(
    context: CommandContext,
    prompter: &P,
    sink: &S,
) -> anyhow::Result<()> {
    log_info!("checking rclone availability...");
    match std::process::Command::new(&context.global.rclone)
        .arg("version")
        .output()
    {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            let first_line = version.lines().next().unwrap_or("unknown");
            sink.success(format!("rclone found: {}", first_line));
        }
        _ => {
            sink.warn(format!(
                "rclone not found at '{}'. Make sure it's installed and accessible.",
                &context.global.rclone
            ));

            sink.plain("you can download it from: https://rclone.org/downloads/");
        }
    }

    let registry_path = context
        .global
        .registry
        .clone()
        .ok_or_else(|| anyhow::anyhow!("registry file not specified"))?;

    sink.info(format!("registry path: {}", registry_path.display()));

    if registry_path.exists() {
        sink.warn("registry file already exists. configuration may already be initialized.");

        let should_continue = prompter
            .confirm("continue anyway?", false)
            .context("failed to prompt confirmation")?;

        if !should_continue {
            sink.info("setup canceled");
            return Ok(());
        }
    }

    let registry = Registry::load(&registry_path).context("failed to load or create registry")?;

    sink.success("registry loaded successfully");
    sink.info(format!(
        "remotes: ({}). paths: ({}).",
        registry.remotes.len(),
        registry.paths.len()
    ));

    Ok(())
}
