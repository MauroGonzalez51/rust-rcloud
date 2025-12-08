mod commands;
mod menu;

use crate::{
    cli::context::CommandContext,
    tui::menu::{CurrentMenu, Menu, MenuState},
};
use crossterm::ExecutableCommand;
use ratatui::{prelude::*, widgets};

pub fn run(context: CommandContext) -> anyhow::Result<()> {
    std::io::stdout().execute(crossterm::terminal::EnterAlternateScreen)?;
    crossterm::terminal::enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
    terminal.clear()?;

    let mut menu_state = MenuState::default();

    loop {
        terminal.draw(|frame| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(frame.area());

            frame.render_widget(
                widgets::Paragraph::new(format!(
                    "Using registry: {}",
                    context.registry.registry_path.display()
                ))
                .block(widgets::Block::default().borders(widgets::Borders::ALL))
                .white()
                .on_blue(),
                layout[0],
            );

            frame.render_stateful_widget(&Menu, layout[1], &mut menu_state);
        })?;

        if crossterm::event::poll(std::time::Duration::from_millis(16))? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                if key.kind == crossterm::event::KeyEventKind::Press {
                    match key.code {
                        crossterm::event::KeyCode::Char('q') => break,
                        crossterm::event::KeyCode::Down => {
                            menu_state.selected_index += 1;
                        }
                        crossterm::event::KeyCode::Up => {
                            if menu_state.selected_index > 0 {
                                menu_state.selected_index -= 1;
                            }
                        }
                        crossterm::event::KeyCode::Enter => {
                            if let CurrentMenu::Main = menu_state.current_menu {
                                let options = commands::Commands::ALL;
                                if let Some(cmd) = options.get(menu_state.selected_index) {
                                    match cmd {
                                        commands::Commands::Path => {
                                            menu_state.current_menu = CurrentMenu::Path;
                                            menu_state.selected_index = 0;
                                        }
                                        commands::Commands::Remote => {
                                            menu_state.current_menu = CurrentMenu::Remote;
                                            menu_state.selected_index = 0;
                                        }
                                        commands::Commands::Sync => {
                                            menu_state.current_menu = CurrentMenu::Sync;
                                            menu_state.selected_index = 0;
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    std::io::stdout().execute(crossterm::terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}
