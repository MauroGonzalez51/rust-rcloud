use crate::{
    cli::{
        commands,
        parser::{Cli, Commands},
    },
    command_context,
    config::prelude::*,
    tui, use_handlers,
    utils::prelude::{LogLevel, Logger, directories, logger},
};
use anyhow::Context;
use clap::{CommandFactory, Parser};
use clap_complete::generate;
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

pub fn run() -> anyhow::Result<(), anyhow::Error> {
    let args = Cli::parse();

    Logger::setup(directories().config_dir.join("rcloud.log"))?;

    if args.global.debug {
        logger().set_level(LogLevel::Debug);
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

    let context = command_context!(app_config, global, registry);

    match &command {
        Some(cmd) => match cmd {
            Commands::Remote { action } => match action {
                commands::remote::command::RemoteCommand::List => {
                    remote_list(context)?;
                }

                commands::remote::command::RemoteCommand::Add { name, provider } => {
                    remote_add(context.with_args(RemoteAddArgs { name, provider }))?;
                }

                commands::remote::command::RemoteCommand::Remove { id } => {
                    remote_remove(context.with_args(RemoteRemoveArgs { id }))?
                }

                commands::remote::command::RemoteCommand::Update { id, name, provider } => {
                    remote_update(context.with_args(RemoteUpdateArgs { id, name, provider }))?;
                }

                commands::remote::command::RemoteCommand::Ls { path, path_config } => {
                    remote_ls(context.with_args(RemoteLsArgs { path, path_config }))?;
                }
            },

            Commands::Path { action } => match action {
                commands::path::command::PathCommand::List => {
                    path_list(context)?;
                }

                commands::path::command::PathCommand::Add {
                    remote_id,
                    local_path,
                    remote_path,
                } => {
                    path_add(context.with_args(PathAddArgs {
                        remote_id,
                        local_path,
                        remote_path,
                    }))?;
                }

                commands::path::command::PathCommand::Remove { id } => {
                    path_remove(context.with_args(PathRemoveArgs { path_id: id }))?;
                }
            },

            Commands::Sync { action } => match action {
                commands::sync::command::SyncCommand::All { tags } => {
                    sync_all(context.with_args(SyncAllArgs { tags }))?;
                }

                commands::sync::command::SyncCommand::Path {
                    direction,
                    path_id,
                    force,
                    clean,
                } => {
                    sync_single(context.with_args(SyncSingleArgs {
                        direction,
                        path_id,
                        force: if *force { &Some(true) } else { &None },
                        clean: if *clean { &Some(true) } else { &None },
                    }))?;
                }
            },

            Commands::Configure => configure_setup(context)?,

            Commands::Completion { shell } => {
                let mut cmd = Cli::command();
                generate(*shell, &mut cmd, "rcloud", &mut io::stdout());
                return Ok(());
            }
        },
        None => tui::run::run_tui(context)?,
    }

    Ok(())
}
