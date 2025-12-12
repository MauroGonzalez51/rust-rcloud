use crate::{
    cli::context::CommandContext,
    command_context,
    tui::commands::{
        PathMenuVariant, RemoteMenuVariant, RootMenu, RootMenuOptions, SyncMenuVariant,
    },
    use_handlers,
};
use crossterm::{execute, terminal};

pub enum ExecutePostOperation {
    None,
}

use_handlers! {
    simple: {
        (path, list),
        (remote, list),
    },
    with_args: {
        (path, add),
        (path, remove),
        (remote, ls),
        (remote, add),
        (remote, remove),
        (remote, update),
        (sync, single),
        (sync, all)
    }
}

pub fn execute(context: CommandContext, action: &RootMenu) -> anyhow::Result<ExecutePostOperation> {
    match action {
        RootMenu::Options(variant) => match variant {
            RootMenuOptions::Exit => {
                execute!(
                    std::io::stdout(),
                    terminal::Clear(terminal::ClearType::All),
                    crossterm::cursor::MoveTo(0, 0)
                )?;

                std::process::exit(0)
            }
        },
        RootMenu::Path(variant) => match variant {
            PathMenuVariant::List => {
                path_list(command_context!(
                    context.config,
                    context.global,
                    context.registry
                ))?;
            }
            PathMenuVariant::Add => {
                path_add(command_context!(
                    context.config,
                    context.global,
                    context.registry,
                    PathAddArgs::default()
                ))?;
            }
            PathMenuVariant::Remove => {
                path_remove(command_context!(
                    context.config,
                    context.global,
                    context.registry,
                    PathRemoveArgs::default()
                ))?;
            }
            _ => unreachable!(),
        },
        RootMenu::Remote(variant) => match variant {
            RemoteMenuVariant::List => {
                remote_list(command_context!(
                    context.config,
                    context.global,
                    context.registry
                ))?;
            }
            RemoteMenuVariant::Ls => {
                remote_ls(command_context!(
                    context.config,
                    context.global,
                    context.registry,
                    RemoteLsArgs::default()
                ))?;
            }
            RemoteMenuVariant::Add => {
                remote_add(command_context!(
                    context.config,
                    context.global,
                    context.registry,
                    RemoteAddArgs::default()
                ))?;
            }
            RemoteMenuVariant::Remove => {
                remote_remove(command_context!(
                    context.config,
                    context.global,
                    context.registry,
                    RemoteRemoveArgs::default()
                ))?;
            }
            RemoteMenuVariant::Update => {
                remote_update(command_context!(
                    context.config,
                    context.global,
                    context.registry,
                    RemoteUpdateArgs::default()
                ))?;
            }
            _ => unreachable!(),
        },
        RootMenu::Sync(variant) => match variant {
            SyncMenuVariant::Single => {
                sync_single(command_context!(
                    context.config,
                    context.global,
                    context.registry,
                    SyncSingleArgs::default()
                ))?;
            }
            SyncMenuVariant::All => {
                sync_all(command_context!(
                    context.config,
                    context.global,
                    context.registry,
                    SyncAllArgs::default()
                ))?;
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }

    Ok(ExecutePostOperation::None)
}
