use crate::cli::commands::{
    path::command::PathCommand, registry::command::RegistryCommand, remote::command::RemoteCommand,
    sync::command::SyncCommand,
};
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Clone, Args)]
pub struct GlobalParameters {
    #[arg(
        short,
        long,
        action = clap::ArgAction::Count,
        help = "Verbose (incremental)", 
        help_heading = "GLOBAL OPTIONS", 
        global = true,
    )]
    pub verbose: u8,

    #[arg(
        short = 'c',
        long = "config",
        value_name = "FILE",
        help = "Config File (.toml)",
        help_heading = "GLOBAL OPTIONS",
        global = true,
        value_parser = clap::value_parser!(PathBuf),
        env = "RUST_RCLOUD_CONFIG"
    )]
    pub config: Option<PathBuf>,

    #[arg(
        short = 'r',
        long = "registry",
        value_name = "FILE",
        help = "Registry File",
        help_heading = "GLOBAL OPTIONS",
        global = true,
        value_parser = clap::value_parser!(PathBuf),
        env = "RUST_RCLOUD_REGISTRY",
    )]
    pub registry: Option<PathBuf>,

    #[arg(
        short = 'd',
        long = "debug",
        help = "Debug Mode",
        help_heading = "GLOBAL OPTIONS",
        global = true,
        env = "RUST_RCLOUD_DEBUG"
    )]
    pub debug: bool,

    #[arg(
        long = "rclone",
        value_name = "RCLONE_PATH",
        help = "Path to rclone executable",
        help_heading = "GLOBAL OPTIONS",
        global = true,
        env = "RCLONE_PATH",
        default_value = "rclone"
    )]
    pub rclone: String,
}

impl From<Cli> for GlobalParameters {
    fn from(value: Cli) -> Self {
        let global = value.global;
        Self { ..global }
    }
}

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "rust-rcloud CLI",
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalParameters,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(about = "Registry related commands")]
    Registry {
        #[command(subcommand)]
        action: RegistryCommand,
    },
    #[command(about = "Manage Remotes")]
    Remote {
        #[command(subcommand)]
        action: RemoteCommand,
    },
    #[command(about = "Manage Paths")]
    Path {
        #[command(subcommand)]
        action: PathCommand,
    },
    #[command(about = "Sync Operations")]
    Sync {
        #[command(subcommand)]
        action: SyncCommand,
    },
    #[command(about = "Configure CLI")]
    Configure,
}
