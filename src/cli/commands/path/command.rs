use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum PathCommand {
    List,
    Add {
        #[arg(long)]
        remote_id: Option<String>,

        #[arg(long)]
        local_path: Option<String>,

        #[arg(long)]
        remote_path: Option<String>,
    },
}
