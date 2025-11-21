use crate::{
    cli::context::CommandContext, config::prelude::Registry, log_info, log_success, log_warn,
};
use anyhow::Context;

pub fn setup(context: CommandContext) -> anyhow::Result<()> {
    log_info!("checking rclone availability...");
    match std::process::Command::new(&context.global.rclone)
        .arg("version")
        .output()
    {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            let first_line = version.lines().next().unwrap_or("unknown");
            log_success!("rclone found: {}", first_line);
        }
        _ => {
            log_warn!(
                "rclone not found at '{}'. Make sure it's installed and accessible.",
                &context.global.rclone
            );

            println!("you can download it from: https://rclone.org/downloads/");
        }
    }

    let registry_path = context
        .global
        .registry
        .clone()
        .ok_or_else(|| anyhow::anyhow!("registry file not specified"))?;

    log_info!("registry path: {}", registry_path.display());

    if registry_path.exists() {
        log_warn!("registry file already exists. configuration may already be initialized.");

        let should_continue = inquire::Confirm::new("continue anyway?")
            .with_default(false)
            .prompt()
            .context("failed to prompt confirmation")?;

        if !should_continue {
            println!("setup canceled");
            return Ok(());
        }
    }

    let registry = Registry::load(&registry_path).context("failed to load or create registry")?;

    log_success!("registry loaded successfully");
    log_info!(
        "remotes: ({}). paths: ({}).",
        registry.remotes.len(),
        registry.paths.len()
    );

    Ok(())
}
