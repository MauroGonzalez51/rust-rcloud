use crate::{
    cli::context::CommandContext,
    command_context,
    tui::commands::{PathMenuVariant, RootMenu, RootMenuOptions},
    use_handlers,
};
use crossterm::{execute, terminal};

pub enum ExecutePostOperation {
    None,
}

use_handlers! {
    simple: {
        (path, list)
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
            _ => todo!(),
        },
        RootMenu::Path(variant) => match variant {
            PathMenuVariant::List => {
                path_list(command_context!(
                    context.config,
                    context.global,
                    context.registry
                ));

                Ok(ExecutePostOperation::None)
            }
            _ => todo!(),
        },
        _ => todo!(),
    }
}
