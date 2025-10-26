use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum RemoteCommand {
    List,
    Add {
        #[arg(long)]
        name: Option<String>,

        #[arg(long)]
        provider: Option<String>,
    },
    Remove {
        #[arg(long)]
        id: Option<String>,
    },
}
