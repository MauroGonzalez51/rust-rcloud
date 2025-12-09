mod cli;
mod config;
mod hooks;
mod tui;
mod utils;

use crate::{
    cli::{
        context::CommandContext,
        parser::{Cli, Commands},
    },
    config::prelude::*,
    utils::prelude::{directories, logger},
};
use anyhow::Context;
use clap::{CommandFactory, Parser};
use clap_complete::generate;
use dotenvy::dotenv;
use std::io;

use_handlers! {
    simple: {
        (remote, list),
        (path, list),
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
        logger().set_level(utils::logger::LogLevel::Debug);
    }

    let config_path = args
        .global
        .config
        .clone()
        .unwrap_or_else(|| directories().config_dir.join("rcloud.toml"));

    let app_config = AppConfig::load(&config_path).unwrap_or_else(|e| {
        if args.global.config.is_none() || config_path.exists() {
            logger().warn(format!("could not load config file: {}", e));
        }

        AppConfig::default()
    });

    let registry_path = args
        .global
        .registry
        .clone()
        .unwrap_or_else(|| directories().config_dir.join("registry.json"));

    if registry_path.is_dir() {
        anyhow::bail!("registry must be a file: {}", registry_path.display());
    }

    let registry = Registry::load(&registry_path).context("failed to load registry")?;

    let Cli { global, command } = args;

    match &command {
        Some(cmd) => match cmd {
            Commands::Remote { action } => match action {
                cli::commands::remote::command::RemoteCommand::List => {
                    remote_list(command_context!(app_config, global, registry));
                }

                cli::commands::remote::command::RemoteCommand::Add { name, provider } => {
                    remote_add(command_context!(
                        app_config,
                        global,
                        registry,
                        RemoteAddArgs { name, provider }
                    ))?
                }

                cli::commands::remote::command::RemoteCommand::Remove { id } => remote_remove(
                    command_context!(app_config, global, registry, RemoteRemoveArgs { id }),
                )?,

                cli::commands::remote::command::RemoteCommand::Update { id, name, provider } => {
                    remote_update(command_context!(
                        app_config,
                        global,
                        registry,
                        RemoteUpdateArgs { id, name, provider }
                    ))?
                }

                cli::commands::remote::command::RemoteCommand::Ls { path, path_config } => {
                    remote_ls(command_context!(
                        app_config,
                        global,
                        registry,
                        RemoteLsArgs { path, path_config }
                    ))?
                }
            },

            Commands::Path { action } => match action {
                cli::commands::path::command::PathCommand::List => {
                    path_list(command_context!(app_config, global, registry));
                }

                cli::commands::path::command::PathCommand::Add {
                    remote_id,
                    local_path,
                    remote_path,
                } => path_add(command_context!(
                    app_config,
                    global,
                    registry,
                    PathAddArgs {
                        remote_id,
                        local_path,
                        remote_path
                    }
                ))?,

                cli::commands::path::command::PathCommand::Remove { id } => path_remove(
                    command_context!(app_config, global, registry, PathRemoveArgs { path_id: id }),
                )?,
            },

            Commands::Sync { action } => match action {
                cli::commands::sync::command::SyncCommand::All {
                    tags,
                    force_all,
                    clean_all,
                } => sync_all(command_context!(
                    app_config,
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
                        app_config,
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

            Commands::Configure => configure_setup(command_context!(app_config, global, registry))?,

            Commands::Completion { shell } => {
                let mut cmd = Cli::command();
                generate(*shell, &mut cmd, "rcloud", &mut io::stdout());
                return Ok(());
            }
        },
        None => tui::run::run_tui(command_context!(app_config, global, registry))?,
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
        logger().with_context(&err);
        std::process::exit(1);
    }
}
