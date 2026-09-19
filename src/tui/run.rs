use crate::{
    cli::context::CommandContext,
    log_warn,
    tui::{
        commands::{RootMenu, RootMenuVariant},
        execute,
        output::{BufferSink, OutputBuffer},
        prompter::RatatuiPrompter,
        utils::prelude::{TreeNodeGetBy, TreeNodeOperations, TreeNodeRef},
        widgets::{output_panel::OutputPanel, tree_menu::TreeMenu},
    },
};
use crossterm::{event, execute as crossterm_execute, terminal};
use ratatui::{
    prelude::{Backend, Constraint, CrosstermBackend, Direction, Layout, Terminal},
    widgets::StatefulWidget,
};
use std::cell::RefCell;
use std::rc::Rc;

/// Copies the whole output buffer (plain text, sections separated) to the
/// system clipboard.
fn yank_all(buffer: &OutputBuffer) -> anyhow::Result<()> {
    let mut text = String::new();
    for section in &buffer.sections {
        if !section.title.is_empty() {
            text.push_str(&format!("── {} ──\n", section.title));
        }
        for line in &section.lines {
            text.push_str(&line.text);
            text.push('\n');
        }
    }

    let mut clipboard = arboard::Clipboard::new()
        .map_err(|e| anyhow::anyhow!("failed to open clipboard: {}", e))?;
    clipboard
        .set_text(text)
        .map_err(|e| anyhow::anyhow!("failed to set clipboard: {}", e))?;
    Ok(())
}

/// Runs one selected action, staying inside raw mode.
///
/// Opens a titled output section for the action, then runs the handler with a
/// [`BufferSink`] so its output is captured into the panel instead of hitting
/// stdout. Returns `false` when the user chose to exit.
fn run_action<B>(
    terminal: &RefCell<Terminal<B>>,
    buffer: &Rc<RefCell<OutputBuffer>>,
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

    // Exit needs no section/handler.
    if matches!(action, RootMenu::Options(_)) {
        let prompter = RatatuiPrompter::new(terminal);
        let sink = BufferSink::new(Rc::clone(buffer));
        return match execute::execute(context.clone(), &action, &prompter, &sink)? {
            execute::ExecutePostOperation::Exit => Ok(false),
            execute::ExecutePostOperation::None => Ok(true),
        };
    }

    // Open a section titled with the action name; the handler just emits lines.
    buffer.borrow_mut().open_section(format!("{}", action));

    let prompter = RatatuiPrompter::new(terminal);
    let sink = BufferSink::new(Rc::clone(buffer));

    match execute::execute(context.clone(), &action, &prompter, &sink) {
        Ok(execute::ExecutePostOperation::Exit) => Ok(false),
        Ok(execute::ExecutePostOperation::None) => Ok(true),
        Err(err) => {
            // Surface the error into the panel rather than tearing down the TUI.
            use crate::cli::output::OutputSink;
            BufferSink::new(Rc::clone(buffer)).error(format!("{:#}", err));
            Ok(true)
        }
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

    let buffer: Rc<RefCell<OutputBuffer>> = Rc::new(RefCell::new(OutputBuffer::default()));
    let mut output_scroll: u16 = 0;
    let mut yank_active = false;

    let keys = context.config.tui.keys.clone();

    let run_result = (|| -> anyhow::Result<()> {
        loop {
            terminal.borrow_mut().draw(|frame| {
                let has_output = !buffer.borrow().is_empty();

                // Dynamic split: give the output panel a third column only when
                // there is output to show.
                let menu_area = if has_output {
                    let cols = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                        .split(frame.area());

                    let panel = OutputPanel::new(&buffer.borrow(), output_scroll, yank_active);
                    frame.render_widget(panel, cols[1]);
                    cols[0]
                } else {
                    frame.area()
                };

                menu.clone()
                    .render(menu_area, frame.buffer_mut(), &mut state);
            })?;

            let event::Event::Key(k) = event::read()? else {
                continue;
            };

            if k.kind != event::KeyEventKind::Press {
                continue;
            }

            // Yank mode intercepts keys first.
            if yank_active {
                match k.code {
                    event::KeyCode::Char('y') => {
                        yank_all(&buffer.borrow())?;
                        yank_active = false;
                    }
                    event::KeyCode::Esc => yank_active = false,
                    _ => {}
                }
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
            let has_output = !buffer.borrow().is_empty();
            let is_right = matches!(code, event::KeyCode::Enter | event::KeyCode::Right)
                || is(&keys.right, code);

            if is(&keys.quit, code) {
                break;
            } else if code == event::KeyCode::Char('y') && has_output {
                yank_active = true;
            } else if matches!(code, event::KeyCode::PageDown) {
                output_scroll = output_scroll.saturating_add(5);
            } else if matches!(code, event::KeyCode::PageUp) {
                output_scroll = output_scroll.saturating_sub(5);
            } else if matches!(code, event::KeyCode::Down) || is(&keys.down, code) {
                menu.navigate_down(&mut current);
            } else if matches!(code, event::KeyCode::Up) || is(&keys.up, code) {
                menu.navigate_up(&mut current);
            } else if matches!(code, event::KeyCode::Left) || is(&keys.left, code) {
                menu.navigate_left(&mut current, &mut state);
            } else if is_right
                && !run_action(
                    &terminal,
                    &buffer,
                    &mut menu,
                    &context,
                    &mut current,
                    &mut state,
                )?
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
