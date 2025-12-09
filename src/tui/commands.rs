use crate::tui::utils::prelude::{TreeBuilder, TreeNodeRef};

#[derive(Clone, Debug, PartialEq)]
enum RootMenu {
    Root(RootMenuVariant),
    Path(PathMenuVariant),
    Remote(RemoteMenuVariant),
    Sync(SyncMenuVariant),
    Options(RootMenuOptions),
}

#[derive(Clone, Debug, PartialEq)]
enum RootMenuOptions {
    Exit,
    GoBack,
}

#[derive(Clone, Debug, PartialEq)]
enum RootMenuVariant {
    Placeholder,
}

#[derive(Clone, Debug, PartialEq)]
enum PathMenuVariant {
    Placeholder,
    List,
    Add,
    Remove,
}

#[derive(Clone, Debug, PartialEq)]
enum RemoteMenuVariant {
    Placeholder,
    List,
    Ls,
    Add,
    Remove,
    Update,
}

#[derive(Clone, Debug, PartialEq)]
enum SyncMenuVariant {
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
                    TreeBuilder::new(RootMenu::Options(RootMenuOptions::GoBack)),
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
                        TreeBuilder::new(RootMenu::Options(RootMenuOptions::GoBack)),
                        TreeBuilder::new(RootMenu::Options(RootMenuOptions::Exit)),
                    ],
                ),
            )
            .child(
                TreeBuilder::new(RootMenu::Sync(SyncMenuVariant::Placeholder)).with_children(vec![
                    TreeBuilder::new(RootMenu::Sync(SyncMenuVariant::Single)),
                    TreeBuilder::new(RootMenu::Sync(SyncMenuVariant::All)),
                    TreeBuilder::new(RootMenu::Options(RootMenuOptions::GoBack)),
                    TreeBuilder::new(RootMenu::Options(RootMenuOptions::Exit)),
                ]),
            )
            .child(TreeBuilder::new(RootMenu::Options(RootMenuOptions::Exit)))
            .build()
    }
}
