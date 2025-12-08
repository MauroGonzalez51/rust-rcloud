pub enum Commands {
    Path,
    Remote,
    Sync,
}

impl Commands {
    pub const ALL: &'static [Commands] = &[Commands::Path, Commands::Remote, Commands::Sync];

    pub fn title() -> &'static str {
        "Main Menu"
    }
}

impl std::fmt::Display for Commands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Commands::Path => write!(f, "Path"),
            Commands::Remote => write!(f, "Remote"),
            Commands::Sync => write!(f, "Sync"),
        }
    }
}

pub enum PathSubcommand {
    Add,
    List,
    Remove,
}

impl PathSubcommand {
    pub const ALL: &'static [PathSubcommand] = &[
        PathSubcommand::Add,
        PathSubcommand::List,
        PathSubcommand::Remove,
    ];

    pub fn title() -> &'static str {
        "Path Menu"
    }
}

impl std::fmt::Display for PathSubcommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathSubcommand::Add => write!(f, "Add Path"),
            PathSubcommand::List => write!(f, "List Paths"),
            PathSubcommand::Remove => write!(f, "Remove Path"),
        }
    }
}

pub enum RemoteSubcommand {
    Add,
    List,
    Ls,
    Remove,
    Update,
}

impl RemoteSubcommand {
    pub const ALL: &'static [RemoteSubcommand] = &[
        RemoteSubcommand::Add,
        RemoteSubcommand::List,
        RemoteSubcommand::Ls,
        RemoteSubcommand::Remove,
        RemoteSubcommand::Update,
    ];

    pub fn title() -> &'static str {
        "Remote Menu"
    }
}

impl std::fmt::Display for RemoteSubcommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RemoteSubcommand::Add => write!(f, "Add Remote"),
            RemoteSubcommand::List => write!(f, "List Remotes"),
            RemoteSubcommand::Ls => write!(f, "List Remote Files"),
            RemoteSubcommand::Remove => write!(f, "Remove Remote"),
            RemoteSubcommand::Update => write!(f, "Update Remote"),
        }
    }
}

pub enum SyncSubcommand {
    Single,
    All,
}

impl SyncSubcommand {
    pub const ALL: &'static [SyncSubcommand] = &[SyncSubcommand::Single, SyncSubcommand::All];

    pub fn title() -> &'static str {
        "Sync Menu"
    }
}

impl std::fmt::Display for SyncSubcommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncSubcommand::Single => write!(f, "Sync Path"),
            SyncSubcommand::All => write!(f, "Sync All Paths based on Tags"),
        }
    }
}
