mod cli;
mod config;
mod utils;

use clap::Parser;
use cli::{commands::remote, parser::Args, parser::Commands};
use config::prelude::*;

use dotenvy::dotenv;

fn main() -> std::io::Result<()> {
    dotenv().ok();

    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    let args = Args::parse();

    let registry_path = match args.registry.clone() {
        Some(path) => {
            if path.is_dir() {
                eprintln!("[ ERROR ] registry must be a file");

                return Err(std::io::Error::new(
                    std::io::ErrorKind::IsADirectory,
                    "[ ERROR ] registry must be a file",
                ));
            }

            path
        }
        None => {
            eprintln!("[ ERROR ] registry file not especified");

            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "[ ERROR ] registry file not specified",
            ));
        }
    };

    let registry = match Registry::load(&registry_path) {
        Ok(value) => value,
        Err(err) => match err {
            RegistryError::Io(err) => return Err(err),
            RegistryError::Serde(err) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("config parse error: {}", err),
                ));
            }
            RegistryError::Custom(err) => {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, err));
            }
        },
    };

    match &args.command {
        Commands::Remote { action } => match action {
            cli::commands::remote::command::RemoteCommand::List => {
                remote::handlers::list::remote_list(&args, &registry)
            }
            cli::commands::remote::command::RemoteCommand::Add { name, provider } => {
                remote::handlers::add::remote_add(&args, &registry, name, provider)
            }
        },
    }

    Ok(())
}
