mod cli;
mod config;
mod utils;

use crate::{
    cli::{
        context::CommandContext,
        parser::{Cli, Commands},
    },
    config::prelude::*,
    utils::logger::LOG,
};
use anyhow::{Context, bail};
use clap::Parser;
use dotenvy::dotenv;

use_handlers! {
    simple: {
        (remote, list),
        (path, list),
        (registry, edit),
        (configure, setup)
    },
    with_args: {
        (remote, add),
        (remote, remove),
        (remote, update),
        (remote, ls),
        (path, add),
        (path, remove),
        (sync, single),
        (sync, all),
    }
}

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
                registry_edit(command_context!(global, registry))?
            }
        },

        Commands::Remote { action } => match action {
            cli::commands::remote::command::RemoteCommand::List => {
                remote_list(command_context!(global, registry));
            }

            cli::commands::remote::command::RemoteCommand::Add { name, provider } => remote_add(
                command_context!(global, registry, RemoteAddArgs { name, provider }),
            )?,

            cli::commands::remote::command::RemoteCommand::Remove { id } => {
                remote_remove(command_context!(global, registry, RemoteRemoveArgs { id }))?
            }

            cli::commands::remote::command::RemoteCommand::Update { id, name, provider } => {
                remote_update(command_context!(
                    global,
                    registry,
                    RemoteUpdateArgs { id, name, provider }
                ))?
            }

            cli::commands::remote::command::RemoteCommand::Ls { path, path_config } => remote_ls(
                command_context!(global, registry, RemoteLsArgs { path, path_config }),
            )?,
        },

        Commands::Path { action } => match action {
            cli::commands::path::command::PathCommand::List => {
                path_list(command_context!(global, registry));
            }

            cli::commands::path::command::PathCommand::Add {
                remote_id,
                local_path,
                remote_path,
            } => path_add(command_context!(
                global,
                registry,
                PathAddArgs {
                    remote_id,
                    local_path,
                    remote_path
                }
            ))?,

            cli::commands::path::command::PathCommand::Remove { id } => path_remove(
                command_context!(global, registry, PathRemoveArgs { path_id: id }),
            )?,
        },

        Commands::Sync { action } => match action {
            cli::commands::sync::command::SyncCommand::All {
                tags,
                force_all,
                clean_all,
            } => sync_all(command_context!(
                global,
                registry,
                SyncAllArgs {
                    tags,
                    force_all,
                    clean_all
                }
            ))?,

            cli::commands::sync::command::SyncCommand::Path {
                direction,
                path_id,
                force,
                clean,
            } => {
                sync_single(command_context!(
                    global,
                    registry,
                    SyncSingleArgs {
                        direction,
                        path_id,
                        force,
                        clean
                    }
                ))?;
            }
        },

        Commands::Configure => configure_setup(command_context!(global, registry))?,
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
