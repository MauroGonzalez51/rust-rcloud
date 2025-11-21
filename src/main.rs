mod cli;
mod config;
mod utils;

use crate::{
    cli::{
        commands,
        context::CommandContext,
        parser::{Cli, Commands},
    },
    config::prelude::*,
    utils::logger::LOG,
};
use anyhow::{Context, bail};
use clap::Parser;
use dotenvy::dotenv;

fn run() -> anyhow::Result<(), anyhow::Error> {
    let args = Cli::parse();

    if args.global.debug {
        LOG.set_level(utils::logger::LogLevel::Debug);
    }

    let registry_path = args
        .global
        .registry
        .clone()
        .ok_or_else(|| anyhow::anyhow!("registry file not especified"))?;

    if registry_path.is_dir() {
        bail!(
            "registry must be a file, not a directory: {}",
            registry_path.display()
        );
    }

    let registry = Registry::load(&registry_path).context("failed to load registry")?;

    let Cli { global, command } = args;

    match &command {
        Commands::Registry { action } => match action {
            cli::commands::registry::command::RegistryCommand::Edit => {
                commands::registry::handlers::edit::registry_edit(CommandContext::from((
                    global, registry,
                )))?
            }
        },

        Commands::Remote { action } => match action {
            cli::commands::remote::command::RemoteCommand::List => {
                commands::remote::handlers::list::remote_list(CommandContext::from((
                    global, registry,
                )))
            }

            cli::commands::remote::command::RemoteCommand::Add { name, provider } => {
                commands::remote::handlers::add::remote_add(CommandContext::from((
                    global,
                    registry,
                    commands::remote::handlers::add::LocalArgs { name, provider },
                )))?
            }

            cli::commands::remote::command::RemoteCommand::Remove { id } => {
                commands::remote::handlers::remove::remote_remove(CommandContext::from((
                    global,
                    registry,
                    commands::remote::handlers::remove::LocalArgs { id },
                )))?
            }

            cli::commands::remote::command::RemoteCommand::Update { id, name, provider } => {
                commands::remote::handlers::update::remote_update(CommandContext::from((
                    global,
                    registry,
                    commands::remote::handlers::update::LocalArgs { id, name, provider },
                )))?
            }

            cli::commands::remote::command::RemoteCommand::Ls { path, path_config } => {
                commands::remote::handlers::ls::remote_ls(CommandContext::from((
                    global,
                    registry,
                    commands::remote::handlers::ls::LocalArgs { path, path_config },
                )))?
            }
        },

        Commands::Path { action } => match action {
            cli::commands::path::command::PathCommand::List => {
                commands::path::handlers::list::path_list(CommandContext::from((global, registry)))
            }

            cli::commands::path::command::PathCommand::Add {
                remote_id,
                local_path,
                remote_path,
            } => commands::path::handlers::add::path_add(CommandContext::from((
                global,
                registry,
                commands::path::handlers::add::LocalArgs {
                    remote_id,
                    local_path,
                    remote_path,
                },
            )))?,

            cli::commands::path::command::PathCommand::Remove { id } => {
                commands::path::handlers::remove::path_remove(CommandContext::from((
                    global,
                    registry,
                    commands::path::handlers::remove::LocalArgs { path_id: id },
                )))?
            }
        },

        Commands::Sync { action } => match action {
            cli::commands::sync::command::SyncCommand::All {
                tags,
                force_all,
                clean_all,
            } => commands::sync::handlers::all_sync::all_sync(CommandContext::from((
                global,
                registry,
                commands::sync::handlers::all_sync::LocalArgs {
                    tags,
                    force_all,
                    clean_all,
                },
            )))?,

            cli::commands::sync::command::SyncCommand::Path {
                direction,
                path_id,
                force,
                clean,
            } => commands::sync::handlers::path_sync::path_sync(CommandContext::from((
                global,
                registry,
                commands::sync::handlers::path_sync::LocalArgs {
                    direction,
                    path_id,
                    force,
                    clean,
                },
            )))?,
        },

        Commands::Configure => {
            commands::configure::setup::setup(CommandContext::from((global, registry)))?
        }
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
