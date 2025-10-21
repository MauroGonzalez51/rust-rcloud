mod cli;
mod config;

use clap::Parser;
use cli::parser::Args;
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

    Ok(())
}
