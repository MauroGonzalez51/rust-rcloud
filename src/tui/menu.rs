use crate::tui::commands;
use ratatui::{prelude::*, widgets};

pub struct MenuState {
    pub selected_index: usize,
    pub current_menu: CurrentMenu,
}

impl Default for MenuState {
    fn default() -> Self {
        Self {
            selected_index: 0,
            current_menu: CurrentMenu::Main,
        }
    }
}

pub enum CurrentMenu {
    Main,
    Path,
    Remote,
    Sync,
}

pub struct Menu;

impl widgets::StatefulWidget for &Menu {
    type State = MenuState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let items: Vec<String> = match state.current_menu {
            CurrentMenu::Main => commands::Commands::ALL
                .iter()
                .map(|variant| format!("{}", variant))
                .collect(),
            CurrentMenu::Path => commands::PathSubcommand::ALL
                .iter()
                .map(|variant| format!("{}", variant))
                .collect(),
            CurrentMenu::Remote => commands::RemoteSubcommand::ALL
                .iter()
                .map(|variant| format!("{}", variant))
                .collect(),
            CurrentMenu::Sync => commands::SyncSubcommand::ALL
                .iter()
                .map(|variant| format!("{}", variant))
                .collect(),
        };

        let items: Vec<widgets::ListItem> = items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                if i == state.selected_index {
                    return widgets::ListItem::new(item.as_str()).style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    );
                }

                widgets::ListItem::new(item.as_str()).style(Style::default().fg(Color::White))
            })
            .collect();

        let title = match state.current_menu {
            CurrentMenu::Main => commands::Commands::title(),
            CurrentMenu::Path => commands::PathSubcommand::title(),
            CurrentMenu::Remote => commands::RemoteSubcommand::title(),
            CurrentMenu::Sync => commands::SyncSubcommand::title(),
        };

        let list = widgets::List::new(items).block(
            widgets::Block::default()
                .title(title)
                .borders(widgets::Borders::ALL),
        );

        widgets::Widget::render(list, area, buf);
    }
}
