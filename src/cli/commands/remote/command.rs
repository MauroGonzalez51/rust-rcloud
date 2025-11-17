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
    Update {
        #[arg(long)]
        id: Option<String>,

        #[arg(long)]
        name: Option<String>,

        #[arg(long)]
        provider: Option<String>,
    },
    Ls {
        #[arg(value_name = "REMOTE_PATH", help = "e.g. drive:documents")]
        path: Option<String>,

        #[arg(
            value_name = "PATH_CONFIG_ID",
            long,
            help = "Use the registry in order to select the path"
        )]
        path_config: Option<String>,
    },
}
