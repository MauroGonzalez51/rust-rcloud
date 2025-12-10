use crate::{
    cli::context::CommandContext,
    log_warn,
    tui::{
        commands::{RootMenu, RootMenuVariant},
        utils::prelude::{TreeNodeGetBy, TreeNodeOperations, TreeNodeRef},
        widgets::tree_menu::TreeMenu,
    },
};
use crossterm::{event, execute, terminal};
use ratatui::{
    prelude::{CrosstermBackend, Terminal},
    widgets::StatefulWidget,
};

pub fn run_tui(_context: CommandContext) -> anyhow::Result<()> {
    terminal::enable_raw_mode()?;

    let mut stdout = std::io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;

    let tree: TreeNodeRef<RootMenu> = RootMenu::Root(RootMenuVariant::Placeholder).into();
    let mut state = tree.borrow().value.clone();
    let mut selected: usize = 0;

    loop {
        terminal.draw(|frame| {
            let widget = TreeMenu::new(tree.clone(), selected);
            widget.render(frame.area(), frame.buffer_mut(), &mut state);
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
                    let len = current.borrow().children().len();

                    if len > 0 {
                        selected = (selected + 1) % len;
                    }
                }
                event::KeyCode::Char('k') | event::KeyCode::Up => {
                    let len = current.borrow().children().len();

                    if len > 0 {
                        selected = (selected + len - 1) % len;
                    }
                }
                event::KeyCode::Char('l') | event::KeyCode::Enter | event::KeyCode::Right => {
                    let child = { current.borrow().children().get(selected).cloned() };

                    if let Some(selected_node) = child {
                        if !selected_node.borrow().children().is_empty() {
                            state = selected_node.borrow().value.clone();
                            selected = 0;
                            continue;
                        }

                        // TODO: handle action here
                        let action = selected_node.borrow().value.clone();
                        log_warn!("execute action: {:?}", action);
                    }
                }
                event::KeyCode::Char('h') | event::KeyCode::Left => {
                    let parent = { current.borrow().parent() };

                    if let Some(new_parent) = parent {
                        let new_selected = new_parent
                            .borrow()
                            .children()
                            .iter()
                            .position(|child| std::rc::Rc::ptr_eq(child, &current))
                            .unwrap_or(0);

                        current = new_parent;
                        state = current.borrow().value.clone();
                        selected = new_selected;
                    }
                }
                _ => {}
            }
        }
    }

    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;

    Ok(())
}
