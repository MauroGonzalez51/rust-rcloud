use crate::{
    cli::context::CommandContext,
    log_debug, log_warn,
    tui::{
        commands::{RootMenu, RootMenuVariant},
        execute,
        utils::prelude::{TreeNodeGetBy, TreeNodeOperations, TreeNodeRef},
        widgets::tree_menu::TreeMenu,
    },
};
use anyhow::Context;
use crossterm::{event, execute, terminal};
use ratatui::{
    prelude::{Backend, CrosstermBackend, Terminal},
    widgets::StatefulWidget,
};

fn execute<B>(
    terminal: &mut Terminal<B>,
    menu: &mut TreeMenu<RootMenu>,
    context: &CommandContext,
    current: &mut TreeNodeRef<RootMenu>,
    state: &mut RootMenu,
) -> anyhow::Result<()>
where
    B: Backend + std::io::Write,
{
    if let Some(action) = menu.navigate_right(current, state) {
        log_debug!("execute action: {:?}", action);

        terminal::disable_raw_mode()?;
        execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;

        match execute::execute(context.clone(), &action)? {
            execute::ExecutePostOperation::None => {
                let should_continue = inquire::Confirm::new("Go back to TUI?")
                    .with_default(true)
                    .prompt()
                    .context("failed to get confirmation")?;

                if !should_continue {
                    std::process::exit(0);
                }

                execute!(
                    std::io::stdout(),
                    terminal::Clear(terminal::ClearType::All),
                )?;

                execute!(terminal.backend_mut(), terminal::EnterAlternateScreen)?;
                terminal::enable_raw_mode()?;
            }
        }
    }

    Ok(())
}

pub fn run_tui(context: CommandContext) -> anyhow::Result<()> {
    terminal::enable_raw_mode()?;

    let mut stdout = std::io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;

    let tree: TreeNodeRef<RootMenu> = RootMenu::Root(RootMenuVariant::Placeholder).into();
    let mut state = tree.borrow().value.clone();
    let mut menu = TreeMenu::new(tree.clone());

    loop {
        terminal.draw(|frame| {
            menu.clone()
                .render(frame.area(), frame.buffer_mut(), &mut state);
        })?;

        if let event::Event::Key(k) = event::read()? {
            if k.kind != event::KeyEventKind::Press {
                continue;
            }

            let mut current = match tree.get(TreeNodeGetBy::Value(state.clone())) {
                Some(current) => current,
                None => {
                    log_warn!("current node not found for state {:?}", state);
                    continue;
                }
            };

            match k.code {
                event::KeyCode::Char(c) if context.config.tui.keys.quit.contains(&c) => break,

                event::KeyCode::Char(c) if context.config.tui.keys.down.contains(&c) => {
                    menu.navigate_down(&mut current);
                }
                event::KeyCode::Down => menu.navigate_down(&mut current),

                event::KeyCode::Char(c) if context.config.tui.keys.up.contains(&c) => {
                    menu.navigate_up(&mut current);
                }
                event::KeyCode::Up => menu.navigate_up(&mut current),

                event::KeyCode::Char(c) if context.config.tui.keys.right.contains(&c) => {
                    execute(&mut terminal, &mut menu, &context, &mut current, &mut state)?;
                }
                event::KeyCode::Enter | event::KeyCode::Right => {
                    execute(&mut terminal, &mut menu, &context, &mut current, &mut state)?;
                }

                event::KeyCode::Char('h') => {
                    menu.navigate_left(&mut current, &mut state);
                }
                event::KeyCode::Left => {
                    menu.navigate_left(&mut current, &mut state);
                }

                _ => {}
            }
        }
    }

    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;

    Ok(())
}
