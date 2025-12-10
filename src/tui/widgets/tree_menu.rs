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

impl<T: Clone + PartialEq + std::fmt::Display> TreeMenu<T> {
    fn layout(
        &self,
        area: &layout::Rect,
        parent: &Option<TreeNodeRef<T>>,
    ) -> std::rc::Rc<[layout::Rect]> {
        match parent {
            Some(_) => layout::Layout::default()
                .direction(layout::Direction::Horizontal)
                .constraints([
                    layout::Constraint::Percentage(30),
                    layout::Constraint::Percentage(30),
                    layout::Constraint::Percentage(40),
                ])
                .split(*area),
            None => layout::Layout::default()
                .direction(layout::Direction::Horizontal)
                .constraints([
                    layout::Constraint::Percentage(60),
                    layout::Constraint::Percentage(40),
                ])
                .split(*area),
        }
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

        let current_idx_in_parent = current.borrow().parent().as_ref().and_then(|parent| {
            parent
                .borrow()
                .children()
                .iter()
                .position(|child| std::rc::Rc::ptr_eq(child, &current))
        });

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

                        if Some(idx) == current_idx_in_parent {
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
        let text_style = Style::default().fg(Color::White);

        let layout = self.layout(&area, &current.borrow().parent());

        let current_items_widget = List::new(current_items)
            .block(Block::default().borders(Borders::ALL).style(border_style))
            .style(text_style);

        let previous_items_widget = List::new(previous_items)
            .block(Block::default().borders(Borders::ALL).style(border_style))
            .style(text_style);

        let execution_widget = Paragraph::new("Execution")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Exec")
                    .style(border_style),
            )
            .style(Style::default().fg(Color::Gray));

        match current.borrow().parent() {
            Some(_) => {
                Widget::render(previous_items_widget, layout[0], buf);

                Widget::render(current_items_widget, layout[1], buf);

                Widget::render(execution_widget, layout[2], buf);
            }
            None => {
                Widget::render(current_items_widget, layout[0], buf);

                Widget::render(execution_widget, layout[1], buf);
            }
        }
    }
}
