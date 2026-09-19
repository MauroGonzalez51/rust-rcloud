//! A [`Prompter`](crate::cli::prompter::Prompter) that gathers input inside the
//! TUI's own raw mode, without leaving the alternate screen.
//!
//! Each method runs a small self-contained event loop that draws the prompt
//! over the current frame and reads keys until the user commits (Enter) or
//! cancels (Esc). This is what lets the TUI reuse the CLI handlers without the
//! disable-raw-mode / re-enable dance.

use crate::cli::prompter::{Prompter, TextOptions};
use anyhow::Context as _;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    prelude::{Backend, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::cell::RefCell;

/// Prompter that draws over an owned ratatui [`Terminal`].
pub struct RatatuiPrompter<'t, B: Backend> {
    terminal: &'t RefCell<Terminal<B>>,
}

impl<'t, B: Backend> RatatuiPrompter<'t, B> {
    /// Wraps a terminal handle for prompting.
    pub fn new(terminal: &'t RefCell<Terminal<B>>) -> Self {
        Self { terminal }
    }

    /// Reads the next key press, ignoring releases/repeats.
    fn next_key() -> anyhow::Result<KeyCode> {
        loop {
            if let Event::Key(k) = event::read().context("failed to read event")?
                && k.kind == KeyEventKind::Press
            {
                return Ok(k.code);
            }
        }
    }

    /// Centered rectangle `pct_x`/`pct_y` percent of `area`.
    fn centered(area: Rect, pct_x: u16, pct_y: u16) -> Rect {
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - pct_y) / 2),
                Constraint::Percentage(pct_y),
                Constraint::Percentage((100 - pct_y) / 2),
            ])
            .split(area);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - pct_x) / 2),
                Constraint::Percentage(pct_x),
                Constraint::Percentage((100 - pct_x) / 2),
            ])
            .split(vertical[1])[1]
    }

    fn bordered<'a>(title: &'a str) -> Block<'a> {
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan))
    }

    /// Shared text/password loop. When `mask` is set the input is hidden.
    fn read_line(
        &self,
        message: &str,
        default: Option<&str>,
        help: Option<&str>,
        required: bool,
        mask: bool,
    ) -> anyhow::Result<String> {
        let mut buffer = default.unwrap_or("").to_string();

        loop {
            self.terminal.borrow_mut().draw(|frame| {
                let area = Self::centered(frame.area(), 60, 20);
                let shown = if mask {
                    "*".repeat(buffer.chars().count())
                } else {
                    buffer.clone()
                };

                let mut lines = vec![Line::from(vec![
                    Span::styled("> ", Style::default().fg(Color::Cyan)),
                    Span::raw(shown),
                ])];
                if let Some(help) = help {
                    lines.push(Line::from(Span::styled(
                        help,
                        Style::default().fg(Color::DarkGray),
                    )));
                }
                lines.push(Line::from(Span::styled(
                    "Enter: confirm   Esc: cancel",
                    Style::default().fg(Color::DarkGray),
                )));

                let paragraph = Paragraph::new(lines).block(Self::bordered(message));
                frame.render_widget(paragraph, area);
            })?;

            match Self::next_key()? {
                KeyCode::Char(c) => buffer.push(c),
                KeyCode::Backspace => {
                    buffer.pop();
                }
                KeyCode::Esc => anyhow::bail!("input cancelled"),
                KeyCode::Enter => {
                    if required && buffer.trim().is_empty() {
                        continue;
                    }
                    return Ok(buffer);
                }
                _ => {}
            }
        }
    }
}

impl<'t, B: Backend> Prompter for RatatuiPrompter<'t, B> {
    fn text(&self, message: &str, options: TextOptions) -> anyhow::Result<String> {
        self.read_line(
            message,
            options.default,
            options.help,
            options.required,
            false,
        )
    }

    fn password(&self, message: &str) -> anyhow::Result<String> {
        self.read_line(message, None, None, true, true)
    }

    fn confirm(&self, message: &str, default: bool) -> anyhow::Result<bool> {
        let mut choice = default;

        loop {
            self.terminal.borrow_mut().draw(|frame| {
                let area = Self::centered(frame.area(), 50, 20);
                let yes = if choice { "[Yes]" } else { " Yes " };
                let no = if choice { " No " } else { "[No]" };
                let lines = vec![
                    Line::from(vec![
                        Span::styled(
                            yes,
                            Style::default()
                                .fg(if choice { Color::Green } else { Color::White })
                                .add_modifier(if choice {
                                    Modifier::BOLD | Modifier::REVERSED
                                } else {
                                    Modifier::empty()
                                }),
                        ),
                        Span::raw("  "),
                        Span::styled(
                            no,
                            Style::default()
                                .fg(if choice { Color::White } else { Color::Red })
                                .add_modifier(if choice {
                                    Modifier::empty()
                                } else {
                                    Modifier::BOLD | Modifier::REVERSED
                                }),
                        ),
                    ]),
                    Line::from(Span::styled(
                        "h/l or ←/→ to toggle   Enter: confirm   y/n",
                        Style::default().fg(Color::DarkGray),
                    )),
                ];
                let paragraph = Paragraph::new(lines).block(Self::bordered(message));
                frame.render_widget(paragraph, area);
            })?;

            match Self::next_key()? {
                KeyCode::Char('y') | KeyCode::Char('Y') => return Ok(true),
                KeyCode::Char('n') | KeyCode::Char('N') => return Ok(false),
                KeyCode::Left | KeyCode::Char('h') | KeyCode::Right | KeyCode::Char('l') => {
                    choice = !choice;
                }
                KeyCode::Enter => return Ok(choice),
                KeyCode::Esc => anyhow::bail!("confirmation cancelled"),
                _ => {}
            }
        }
    }

    fn select<T>(&self, message: &str, options: Vec<T>) -> anyhow::Result<T>
    where
        T: std::fmt::Display,
    {
        anyhow::ensure!(!options.is_empty(), "no options to select from");
        let labels: Vec<String> = options.iter().map(|o| o.to_string()).collect();
        let mut selected = 0usize;

        loop {
            self.terminal.borrow_mut().draw(|frame| {
                let area = Self::centered(frame.area(), 60, 60);
                let items: Vec<ListItem> = labels
                    .iter()
                    .enumerate()
                    .map(|(i, label)| {
                        let style = if i == selected {
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD | Modifier::REVERSED)
                        } else {
                            Style::default().fg(Color::White)
                        };
                        ListItem::new(label.clone()).style(style)
                    })
                    .collect();
                let list = List::new(items).block(Self::bordered(message));
                frame.render_widget(list, area);
            })?;

            match Self::next_key()? {
                KeyCode::Down | KeyCode::Char('j') => {
                    selected = (selected + 1) % labels.len();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    selected = (selected + labels.len() - 1) % labels.len();
                }
                KeyCode::Enter => {
                    let mut remaining = options;
                    return Ok(remaining.swap_remove(selected));
                }
                KeyCode::Esc => anyhow::bail!("selection cancelled"),
                _ => {}
            }
        }
    }

    fn multi_select<T>(
        &self,
        message: &str,
        options: Vec<T>,
        defaults: &[usize],
    ) -> anyhow::Result<Vec<T>>
    where
        T: std::fmt::Display,
    {
        let labels: Vec<String> = options.iter().map(|o| o.to_string()).collect();
        let mut checked: Vec<bool> = (0..options.len())
            .map(|i| defaults.contains(&i))
            .collect();
        let mut cursor = 0usize;

        loop {
            self.terminal.borrow_mut().draw(|frame| {
                let area = Self::centered(frame.area(), 60, 60);
                let items: Vec<ListItem> = labels
                    .iter()
                    .enumerate()
                    .map(|(i, label)| {
                        let marker = if checked[i] { "[x] " } else { "[ ] " };
                        let style = if i == cursor {
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD | Modifier::REVERSED)
                        } else {
                            Style::default().fg(Color::White)
                        };
                        ListItem::new(format!("{}{}", marker, label)).style(style)
                    })
                    .collect();
                let block = Self::bordered(message).title_bottom(
                    "space: toggle   Enter: confirm   Esc: cancel",
                );
                let list = List::new(items).block(block);
                frame.render_widget(list, area);
            })?;

            match Self::next_key()? {
                KeyCode::Down | KeyCode::Char('j') if !labels.is_empty() => {
                    cursor = (cursor + 1) % labels.len();
                }
                KeyCode::Up | KeyCode::Char('k') if !labels.is_empty() => {
                    cursor = (cursor + labels.len() - 1) % labels.len();
                }
                KeyCode::Char(' ') if !labels.is_empty() => {
                    checked[cursor] = !checked[cursor];
                }
                KeyCode::Enter => {
                    let result = options
                        .into_iter()
                        .zip(checked)
                        .filter_map(|(opt, keep)| keep.then_some(opt))
                        .collect();
                    return Ok(result);
                }
                KeyCode::Esc => anyhow::bail!("selection cancelled"),
                _ => {}
            }
        }
    }
}
