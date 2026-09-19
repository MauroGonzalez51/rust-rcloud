use crate::{
    cli::output::OutputLevel,
    tui::output::OutputBuffer,
};
use ratatui::{
    prelude::{Buffer, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Widget, Wrap},
};

/// Renders the accumulated [`OutputBuffer`] as a scrollable, sectioned panel.
///
/// Each section shows a highlighted title row (the action name) followed by its
/// lines, styled by [`OutputLevel`]. `scroll` is the first visible line offset;
/// `yank_active` toggles a copy-mode border hint.
pub struct OutputPanel<'a> {
    buffer: &'a OutputBuffer,
    scroll: u16,
    yank_active: bool,
}

impl<'a> OutputPanel<'a> {
    pub fn new(buffer: &'a OutputBuffer, scroll: u16, yank_active: bool) -> Self {
        Self {
            buffer,
            scroll,
            yank_active,
        }
    }

    fn level_style(level: OutputLevel) -> Style {
        match level {
            OutputLevel::Plain => Style::default().fg(Color::White),
            OutputLevel::Info => Style::default().fg(Color::Cyan),
            OutputLevel::Success => Style::default().fg(Color::Green),
            OutputLevel::Warn => Style::default().fg(Color::Yellow),
            OutputLevel::Error => Style::default().fg(Color::Red),
        }
    }

    /// Flattens the buffer into styled lines (titles + content).
    fn lines(&self) -> Vec<Line<'a>> {
        let mut out = Vec::new();
        for section in &self.buffer.sections {
            let title = if section.title.is_empty() {
                "── output ──".to_string()
            } else {
                format!("── {} ──", section.title)
            };
            out.push(Line::from(Span::styled(
                title,
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            )));
            for line in &section.lines {
                out.push(Line::from(Span::styled(
                    line.text.clone(),
                    Self::level_style(line.level),
                )));
            }
        }
        out
    }
}

impl Widget for OutputPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (border_color, title) = if self.yank_active {
            (Color::Yellow, " Output — YANK (y: all, Esc: cancel) ")
        } else {
            (Color::DarkGray, " Output (y: yank) ")
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color));

        let paragraph = Paragraph::new(self.lines())
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((self.scroll, 0));

        paragraph.render(area, buf);
    }
}
