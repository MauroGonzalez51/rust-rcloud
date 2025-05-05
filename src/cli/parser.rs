use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "RCloud")]
#[command(about="A CLI for managing RCloud", long_about=None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Remote {
        #[command(subcommand)]
        action: RemoteCommand,
    },
    Path {
        #[command(subcommand)]
        action: PathCommand,
    },
    Sync {
        #[arg(short, long)]
        path_id: String,
    },
    Pull {
        #[arg(short, long)]
        path_id: String,
    },
}

#[derive(Subcommand)]
pub enum RemoteCommand {
    List,
    Add {
        #[arg(short, long)]
        name: String,

        #[arg(short, long)]
        provider: String,
    },
    Remove {
        id: String,
    },
    Update {
        #[arg(short, long)]
        id: String,

        #[arg(short = 'n', long)]
        new_name: Option<String>,

        #[arg(short = 'p', long)]
        new_provider: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum PathCommand {
    List,
    Add {
        #[arg(short = 'r', long)]
        remote_id: String,

        #[arg(short = 'l', long)]
        local_path: String,

        #[arg(short = 'e', long)]
        remote_path: String,
    },
    Remove {
        id: String,
    },
}
