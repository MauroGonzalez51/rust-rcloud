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
    prelude::{CrosstermBackend, Terminal},
    widgets::StatefulWidget,
};

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
                event::KeyCode::Char('q') => break,
                event::KeyCode::Char('j') | event::KeyCode::Down => {
                    menu.navigate_down(&mut current);
                }
                event::KeyCode::Char('k') | event::KeyCode::Up => {
                    menu.navigate_up(&mut current);
                }
                event::KeyCode::Char('l') | event::KeyCode::Enter | event::KeyCode::Right => {
                    if let Some(action) = menu.navigate_right(&mut current, &mut state) {
                        log_debug!("execute action: {:?}", action);

                        terminal::disable_raw_mode()?;
                        execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;

                        match execute::execute(context.clone(), &action)? {
                            execute::ExecutePostOperation::None => {
                                let should_continue = inquire::Confirm::new("Continue?")
                                    .with_default(true)
                                    .prompt()
                                    .context("failed to get confirmation")?;

                                if !should_continue {
                                    std::process::exit(0);
                                }

                                execute!(
                                    std::io::stdout(),
                                    terminal::Clear(terminal::ClearType::All),
                                    crossterm::cursor::MoveTo(0, 0)
                                )?;

                                execute!(terminal.backend_mut(), terminal::EnterAlternateScreen)?;
                                terminal::enable_raw_mode()?;
                            }
                        }
                    }
                }
                event::KeyCode::Char('h') | event::KeyCode::Left => {
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
