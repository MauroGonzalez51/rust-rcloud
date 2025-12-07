use crate::cli::context::CommandContext;
use crossterm::ExecutableCommand;
use ratatui::{
    prelude::{CrosstermBackend, Stylize, Terminal},
    widgets::{Block, Borders, Paragraph},
};

pub fn run(context: CommandContext) -> anyhow::Result<()> {
    std::io::stdout().execute(crossterm::terminal::EnterAlternateScreen)?;
    crossterm::terminal::enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
    terminal.clear()?;

    loop {
        terminal.draw(|frame| {
            let area = frame.area();
            let text = format!(
                "Bienvenido a RCloud TUI\nRegistry: {}\nPresiona 'q' para salir.",
                context.registry.registry_path.display()
            );

            let p = Paragraph::new(text)
                .block(Block::default().title("RCloud").borders(Borders::ALL))
                .white()
                .on_blue();

            frame.render_widget(p, area);
        })?;

        if crossterm::event::poll(std::time::Duration::from_millis(16))? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                if key.kind == crossterm::event::KeyEventKind::Press
                    && key.code == crossterm::event::KeyCode::Char('q')
                {
                    break;
                }
            }
        }
    }

    std::io::stdout().execute(crossterm::terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}
