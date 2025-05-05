mod cli;
mod config;

use clap::Parser;
use cli::parser::{Cli, Commands, RemoteCommand};
use config::schema::{Config, Remote};
use dotenvy::dotenv;
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
    }
}
