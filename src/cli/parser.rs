use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "RCloud")]
#[command(about = "A CLI for managing RCloud", long_about=None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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

    #[command(about = "Sync to a Given Remote")]
    Sync {
        #[arg(short, long)]
        path_id: Option<String>,

        #[arg(short, long)]
        alias: Option<String>,
    },

    #[command(about = "Pull from a Given Remote")]
    Pull {
        #[arg(short, long)]
        path_id: Option<String>,

        #[arg(short, long)]
        alias: Option<String>,
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
    Find {
        #[arg(short, long)]
        id: Option<String>,

        #[arg(short, long)]
        name: Option<String>,

        #[arg(long, help = "Use OR logic for filtering (default is AND)")]
        or: bool
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

        #[arg(short, long)]
        alias: String,
    },
    Remove {
        id: String,
    },
}
