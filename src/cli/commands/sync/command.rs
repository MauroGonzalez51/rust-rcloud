use clap::Subcommand;

use crate::config::prelude::HookExecType;

#[derive(Debug, Subcommand)]
pub enum SyncCommand {
    #[command(about = "Sync all paths")]
    All {
        #[arg(long, value_name = "...TAGS", help = "comma separated tags to sync")]
        tags: Vec<String>,
    },
    #[command(about = "Sync a specific path by ID")]
    Path {
        #[arg(value_name = "PATH_ID", help = "ID of the path to sync")]
        path_id: Option<String>,

        #[arg(long, value_enum, help = "Sync direction")]
        direction: Option<HookExecType>,

        #[arg(short = 'F', long, help = "Force sending to remote")]
        force: bool,

        #[arg(
            short = 'C',
            long,
            help = "Clean target directory before executing workflow"
        )]
        clean: bool,
    },
}
