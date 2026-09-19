use crate::{
    cli::context::CommandContext,
    log_debug, log_warn,
    tui::{
        commands::{RootMenu, RootMenuVariant},
        execute,
        prompter::RatatuiPrompter,
        utils::prelude::{TreeNodeGetBy, TreeNodeOperations, TreeNodeRef},
        widgets::tree_menu::TreeMenu,
    },
};
use crossterm::{event, execute as crossterm_execute, terminal};
use ratatui::{
    prelude::{Backend, CrosstermBackend, Terminal},
    widgets::StatefulWidget,
};
use std::cell::RefCell;

/// Runs one selected action, staying inside raw mode.
///
/// Handlers gather any input they need through the injected
/// [`RatatuiPrompter`], so there is no longer a disable-raw-mode /
/// re-enable-raw-mode dance around each action. Returns `false` when the user
/// chose to exit.
fn run_action<B>(
    terminal: &RefCell<Terminal<B>>,
    menu: &mut TreeMenu<RootMenu>,
    context: &CommandContext,
    current: &mut TreeNodeRef<RootMenu>,
    state: &mut RootMenu,
) -> anyhow::Result<bool>
where
    B: Backend,
{
    let Some(action) = menu.navigate_right(current, state) else {
        return Ok(true);
    };

    log_debug!("execute action: {:?}", action);

    let prompter = RatatuiPrompter::new(terminal);

    match execute::execute(context.clone(), &action, &prompter)? {
        execute::ExecutePostOperation::Exit => Ok(false),
        execute::ExecutePostOperation::None => Ok(true),
    }
}

pub fn run_tui(context: CommandContext) -> anyhow::Result<()> {
    terminal::enable_raw_mode()?;

    let mut stdout = std::io::stdout();
    crossterm_execute!(stdout, terminal::EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let terminal = RefCell::new(Terminal::new(backend)?);

    let tree: TreeNodeRef<RootMenu> = RootMenu::Root(RootMenuVariant::Placeholder).into();
    let mut state = tree.borrow().value.clone();
    let mut menu = TreeMenu::new(tree.clone());

    let keys = &context.config.tui.keys;

    let run_result = (|| -> anyhow::Result<()> {
        loop {
            terminal.borrow_mut().draw(|frame| {
                menu.clone()
                    .render(frame.area(), frame.buffer_mut(), &mut state);
            })?;

            let event::Event::Key(k) = event::read()? else {
                continue;
            };

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

            let is = |set: &[char], code: event::KeyCode| match code {
                event::KeyCode::Char(c) => set.contains(&c),
                _ => false,
            };

            let code = k.code;
            let is_right = matches!(code, event::KeyCode::Enter | event::KeyCode::Right)
                || is(&keys.right, code);

            if is(&keys.quit, code) {
                break;
            } else if matches!(code, event::KeyCode::Down) || is(&keys.down, code) {
                menu.navigate_down(&mut current);
            } else if matches!(code, event::KeyCode::Up) || is(&keys.up, code) {
                menu.navigate_up(&mut current);
            } else if matches!(code, event::KeyCode::Left) || is(&keys.left, code) {
                menu.navigate_left(&mut current, &mut state);
            } else if is_right
                && !run_action(&terminal, &mut menu, &context, &mut current, &mut state)?
            {
                break;
            }
        }
        Ok(())
    })();

    // Always restore the terminal, even if the loop errored.
    terminal::disable_raw_mode()?;
    crossterm_execute!(terminal.borrow_mut().backend_mut(), terminal::LeaveAlternateScreen)?;

    run_result
}
