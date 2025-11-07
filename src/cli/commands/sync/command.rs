use clap::Subcommand;

use crate::config::prelude::HookExecType;

#[derive(Debug, Subcommand)]
pub enum SyncCommand {
    #[command(about = "Sync all paths")]
    All,

    #[command(about = "Sync a specific path by ID")]
    Path {
        #[arg(value_name = "PATH_ID", help = "ID of the path to sync")]
        path_id: Option<String>,

        #[arg(short = 'D', long, value_enum, help = "Sync direction")]
        direction: HookExecType,
    },
}
