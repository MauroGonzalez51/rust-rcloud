use crate::{
    log_warn,
    tui::utils::prelude::{TreeNodeGetBy, TreeNodeOperations, TreeNodeRef},
};
use ratatui::{
    layout,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Widget},
};

pub struct TreeMenu<T: Clone + PartialEq + std::fmt::Display> {
    tree: TreeNodeRef<T>,
    selected: usize,
}

impl<T: Clone + PartialEq + std::fmt::Display> TreeMenu<T> {
    pub fn new(tree: TreeNodeRef<T>, selected: usize) -> Self {
        Self { tree, selected }
    }
}

impl<T: Clone + PartialEq + std::fmt::Display> ratatui::widgets::StatefulWidget for TreeMenu<T> {
    type State = T;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let Some(current) = self.tree.get(TreeNodeGetBy::Value(state.clone())) else {
            log_warn!("current node not found in tree");
            return;
        };

        let has_parent = current.borrow().parent().is_some();

        let previous_items: Vec<ListItem> = current
            .borrow()
            .parent()
            .as_ref()
            .map(|parent| {
                parent
                    .borrow()
                    .children()
                    .iter()
                    .enumerate()
                    .map(|(idx, child)| {
                        let value = format!("{}", child.borrow().value);

                        if idx == self.selected {
                            return ListItem::new(value).style(
                                Style::default()
                                    .fg(Color::Cyan)
                                    .add_modifier(Modifier::BOLD | Modifier::REVERSED),
                            );
                        }

                        ListItem::new(value).style(Style::default())
                    })
                    .collect()
            })
            .unwrap_or_default();

        let current_items: Vec<ListItem> = current
            .borrow()
            .children()
            .iter()
            .enumerate()
            .map(|(idx, child)| {
                let value = format!("{}", child.borrow().value);

                if idx == self.selected {
                    return ListItem::new(value).style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD | Modifier::REVERSED),
                    );
                }

                ListItem::new(value).style(Style::default())
            })
            .collect();

        let border_style = Style::default().fg(Color::DarkGray);

        match has_parent {
            true => {
                let layout = layout::Layout::default()
                    .direction(layout::Direction::Horizontal)
                    .constraints([
                        layout::Constraint::Percentage(30),
                        layout::Constraint::Percentage(30),
                        layout::Constraint::Percentage(40),
                    ])
                    .split(area);

                Widget::render(
                    List::new(previous_items).block(Block::default().borders(Borders::ALL)),
                    layout[0],
                    buf,
                );

                Widget::render(
                    List::new(current_items).block(Block::default().borders(Borders::ALL)),
                    layout[1],
                    buf,
                );

                Widget::render(
                    Paragraph::new("Execution")
                        .block(Block::default().borders(Borders::ALL).title("Exec")),
                    layout[2],
                    buf,
                );
            }
            false => {
                let layout = layout::Layout::default()
                    .direction(layout::Direction::Horizontal)
                    .constraints([
                        layout::Constraint::Percentage(40),
                        layout::Constraint::Percentage(60),
                    ])
                    .split(area);

                Widget::render(
                    List::new(previous_items)
                        .block(Block::default().borders(Borders::ALL).title("Previous")),
                    layout[0],
                    buf,
                );

                Widget::render(
                    List::new(current_items)
                        .block(Block::default().borders(Borders::ALL).title("Current")),
                    layout[1],
                    buf,
                );
            }
        }
    }
}
