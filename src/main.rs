mod cli;
mod config;

use clap::Parser;
use cli::parser::{Cli, Commands, PathCommand, RemoteCommand};
use config::schema::{Config, PathConfig, Remote};
use dotenvy::dotenv;
use std::path::Path;
use uuid::Uuid;

fn main() {
    dotenv().ok();

    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    let config_path = std::env::var("CONFIG_PATH").expect("CONFIG_PATH is not set");
    let mut config = Config::load(config_path.as_str());
    let cli = Cli::parse();

    match cli.command {
        Commands::Remote { action } => match action {
            RemoteCommand::List => {
                let max = config
                    .remotes
                    .iter()
                    .map(|remote| remote.remote_name.len())
                    .max()
                    .unwrap_or(20);

                for remote in config.remotes.iter() {
                    println!(
                        "|{}| {:<width$} ({})",
                        remote.id,
                        remote.remote_name,
                        remote.provider,
                        width = max
                    );
                }
            }
            RemoteCommand::Add { name, provider } => {
                config.remotes.push(Remote {
                    id: Uuid::new_v4().to_string(),
                    remote_name: name,
                    provider,
                });
                config.save();
            }
            RemoteCommand::Remove { id } => {
                config.remotes.retain(|remote| remote.id != id);
                config.save()
            }
            RemoteCommand::Update {
                id,
                new_name,
                new_provider,
            } => {
                if let Some(remote) = config.remotes.iter_mut().find(|remote| remote.id == id) {
                    if let Some(name) = new_name {
                        remote.remote_name = name;
                    }

                    if let Some(provider) = new_provider {
                        remote.provider = provider
                    }

                    config.save();
                    return;
                }

                println!("[ ERROR ] remote with id |{}| not found", id);
            }
        },
        Commands::Path { action } => match action {
            PathCommand::List => {
                let max = config
                    .paths
                    .iter()
                    .map(|path| path.local_path.len())
                    .max()
                    .unwrap_or(20);

                for path in config.paths.iter() {
                    println!(
                        "|{}| {:<width$} -> {} ({})",
                        path.id,
                        path.local_path,
                        path.remote_path,
                        path.remote_id,
                        width = max
                    );
                }
            }
            PathCommand::Add {
                remote_id,
                local_path,
                remote_path,
            } => {
                if !config.remotes.iter().any(|remote| remote.id == remote_id) {
                    println!("[ ERROR ] remote with id |{}| not found", remote_id);
                    return;
                }

                if !Path::new(&local_path).exists() {
                    println!("[ ERROR ] local path |{}| does not exist", local_path);
                    return;
                }

                config.paths.push(PathConfig {
                    id: Uuid::new_v4().to_string(),
                    remote_id,
                    local_path,
                    remote_path,
                });

                config.save();
            }
            PathCommand::Remove { id } => {
                config.paths.retain(|path| path.id != id);
                config.save();
            }
        },
    }
}
