use crate::tui::utils::prelude::{TreeBuilder, TreeNodeRef};

#[derive(Clone, Debug, PartialEq)]
pub enum RootMenu {
    Root(RootMenuVariant),
    Path(PathMenuVariant),
    Remote(RemoteMenuVariant),
    Sync(SyncMenuVariant),
    Options(RootMenuOptions),
}

#[derive(Clone, Debug, PartialEq)]
pub enum RootMenuOptions {
    Exit,
    GoBack,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RootMenuVariant {
    Placeholder,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PathMenuVariant {
    Placeholder,
    List,
    Add,
    Remove,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RemoteMenuVariant {
    Placeholder,
    List,
    Ls,
    Add,
    Remove,
    Update,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SyncMenuVariant {
    Placeholder,
    Single,
    All,
}

impl From<RootMenu> for TreeNodeRef<RootMenu> {
    fn from(_val: RootMenu) -> Self {
        TreeBuilder::new(RootMenu::Root(RootMenuVariant::Placeholder))
            .child(
                TreeBuilder::new(RootMenu::Path(PathMenuVariant::Placeholder)).with_children(vec![
                    TreeBuilder::new(RootMenu::Path(PathMenuVariant::List)),
                    TreeBuilder::new(RootMenu::Path(PathMenuVariant::Add)),
                    TreeBuilder::new(RootMenu::Path(PathMenuVariant::Remove)),
                    TreeBuilder::new(RootMenu::Options(RootMenuOptions::Exit)),
                ]),
            )
            .child(
                TreeBuilder::new(RootMenu::Remote(RemoteMenuVariant::Placeholder)).with_children(
                    vec![
                        TreeBuilder::new(RootMenu::Remote(RemoteMenuVariant::List)),
                        TreeBuilder::new(RootMenu::Remote(RemoteMenuVariant::Ls)),
                        TreeBuilder::new(RootMenu::Remote(RemoteMenuVariant::Add)),
                        TreeBuilder::new(RootMenu::Remote(RemoteMenuVariant::Remove)),
                        TreeBuilder::new(RootMenu::Remote(RemoteMenuVariant::Update)),
                        TreeBuilder::new(RootMenu::Options(RootMenuOptions::Exit)),
                    ],
                ),
            )
            .child(
                TreeBuilder::new(RootMenu::Sync(SyncMenuVariant::Placeholder)).with_children(vec![
                    TreeBuilder::new(RootMenu::Sync(SyncMenuVariant::Single)),
                    TreeBuilder::new(RootMenu::Sync(SyncMenuVariant::All)),
                    TreeBuilder::new(RootMenu::Options(RootMenuOptions::Exit)),
                ]),
            )
            .child(TreeBuilder::new(RootMenu::Options(RootMenuOptions::Exit)))
            .build()
    }
}

impl std::fmt::Display for RootMenu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RootMenu::Root(variant) => match variant {
                RootMenuVariant::Placeholder => write!(f, "Main Menu"),
            },
            RootMenu::Path(variant) => match variant {
                PathMenuVariant::Placeholder => write!(f, "Path Menu"),
                PathMenuVariant::List => write!(f, "List Paths"),
                PathMenuVariant::Add => write!(f, "Add Path"),
                PathMenuVariant::Remove => write!(f, "Remove Path"),
            },
            RootMenu::Remote(variant) => match variant {
                RemoteMenuVariant::Placeholder => write!(f, "Remote Menu"),
                RemoteMenuVariant::List => write!(f, "List Remote"),
                RemoteMenuVariant::Ls => write!(f, "List files in a given Path (Remote)"),
                RemoteMenuVariant::Add => write!(f, "Add Remote"),
                RemoteMenuVariant::Update => write!(f, "Update Remote Information"),
                RemoteMenuVariant::Remove => write!(f, "Remove a Configured Remote"),
            },
            RootMenu::Sync(variant) => match variant {
                SyncMenuVariant::Placeholder => write!(f, "Sync Menu"),
                SyncMenuVariant::Single => write!(f, "Sync Path"),
                SyncMenuVariant::All => write!(f, "Sync ALL paths that matches tags"),
            },
            RootMenu::Options(variant) => match variant {
                RootMenuOptions::GoBack => write!(f, "Go Back"),
                RootMenuOptions::Exit => write!(f, "Exit"),
            },
        }
    }
}
