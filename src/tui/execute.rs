use crate::{
    cli::{context::CommandContext, output::OutputSink, prompter::Prompter},
    command_context,
    tui::commands::{
        PathMenuVariant, RemoteMenuVariant, RootMenu, RootMenuOptions, SyncMenuVariant,
    },
    use_handlers,
};

pub enum ExecutePostOperation {
    None,
    Exit,
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

pub fn execute<P: Prompter, S: OutputSink>(
    context: CommandContext,
    action: &RootMenu,
    prompter: &P,
    sink: &S,
) -> anyhow::Result<ExecutePostOperation> {
    match action {
        RootMenu::Options(variant) => match variant {
            RootMenuOptions::Exit => return Ok(ExecutePostOperation::Exit),
        },
        RootMenu::Path(variant) => match variant {
            PathMenuVariant::List => {
                path_list(
                    command_context!(context.config, context.global, context.registry),
                    sink,
                )?;
            }
            PathMenuVariant::Add => {
                path_add(
                    command_context!(
                        context.config,
                        context.global,
                        context.registry,
                        PathAddArgs::default()
                    ),
                    prompter,
                    sink,
                )?;
            }
            PathMenuVariant::Remove => {
                path_remove(
                    command_context!(
                        context.config,
                        context.global,
                        context.registry,
                        PathRemoveArgs::default()
                    ),
                    prompter,
                    sink,
                )?;
            }
            _ => unreachable!(),
        },
        RootMenu::Remote(variant) => match variant {
            RemoteMenuVariant::List => {
                remote_list(
                    command_context!(context.config, context.global, context.registry),
                    sink,
                )?;
            }
            RemoteMenuVariant::Ls => {
                remote_ls(
                    command_context!(
                        context.config,
                        context.global,
                        context.registry,
                        RemoteLsArgs::default()
                    ),
                    prompter,
                    sink,
                )?;
            }
            RemoteMenuVariant::Add => {
                remote_add(
                    command_context!(
                        context.config,
                        context.global,
                        context.registry,
                        RemoteAddArgs::default()
                    ),
                    prompter,
                    sink,
                )?;
            }
            RemoteMenuVariant::Remove => {
                remote_remove(
                    command_context!(
                        context.config,
                        context.global,
                        context.registry,
                        RemoteRemoveArgs::default()
                    ),
                    prompter,
                    sink,
                )?;
            }
            RemoteMenuVariant::Update => {
                remote_update(
                    command_context!(
                        context.config,
                        context.global,
                        context.registry,
                        RemoteUpdateArgs::default()
                    ),
                    prompter,
                    sink,
                )?;
            }
            _ => unreachable!(),
        },
        RootMenu::Sync(variant) => match variant {
            SyncMenuVariant::Single => {
                sync_single(
                    command_context!(
                        context.config,
                        context.global,
                        context.registry,
                        SyncSingleArgs::default()
                    ),
                    prompter,
                    sink,
                )?;
            }
            SyncMenuVariant::All => {
                sync_all(
                    command_context!(
                        context.config,
                        context.global,
                        context.registry,
                        SyncAllArgs::default()
                    ),
                    prompter,
                    sink,
                )?;
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }

    Ok(ExecutePostOperation::None)
}
