mod cli;
mod config;
mod utils;

use crate::{
    cli::{commands, parser::Args, parser::Commands},
    config::prelude::*,
    utils::logger::LOG,
};
use anyhow::{Context, bail};
use clap::Parser;
use dotenvy::dotenv;

fn run() -> anyhow::Result<(), anyhow::Error> {
    let args = Args::parse();

    let registry_path = args
        .registry
        .clone()
        .ok_or_else(|| anyhow::anyhow!("registry file not especified"))?;

    if registry_path.is_dir() {
        bail!(
            "registry must be a file, not a directory: {}",
            registry_path.display()
        );
    }

    let mut registry = Registry::load(&registry_path).context("failed to load registry")?;

    match &args.command {
        Commands::Remote { action } => match action {
            cli::commands::remote::command::RemoteCommand::List => {
                commands::remote::handlers::list::remote_list(&args, &registry)
            }
            cli::commands::remote::command::RemoteCommand::Add { name, provider } => {
                commands::remote::handlers::add::remote_add(&args, &mut registry, name, provider)?
            }
            cli::commands::remote::command::RemoteCommand::Remove { id } => {
                commands::remote::handlers::remove::remote_remove(&args, &mut registry, id)?
            }
            cli::commands::remote::command::RemoteCommand::Update { id, name, provider } => {
                commands::remote::handlers::update::remote_update(
                    &args,
                    &mut registry,
                    id,
                    name,
                    provider,
                )?
            }
        },
        Commands::Path { action } => match action {
            cli::commands::path::command::PathCommand::List => {
                commands::path::handlers::list::path_list(&args, &registry)
            }
            cli::commands::path::command::PathCommand::Add {
                remote_id,
                local_path,
                remote_path,
            } => commands::path::handlers::add::path_add(
                &args,
                &registry,
                remote_id,
                local_path,
                remote_path,
            )?,
        },
    }

    Ok(())
}

fn main() {
    dotenv().ok();

    #[cfg(debug_assertions)]
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    if let Err(err) = run() {
        LOG.with_context(&err);
        std::process::exit(1);
    }
}
