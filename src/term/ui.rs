use ratatui::{
    prelude::*,
    widgets::{
        block::Title, Block, Borders, Clear, List, ListItem, ListState, Padding, Paragraph,
        Scrollbar, ScrollbarOrientation::VerticalRight, ScrollbarState, Wrap,
    },
};

use crate::types::LibraryItem;

use super::app::App;

#[derive(Debug, Default)]
pub struct StatefulListCounter {
    state: ListState,
    size: usize,
}

impl StatefulListCounter {
    pub fn new(size: usize) -> Self {
        StatefulListCounter {
            state: ListState::default(),
            size,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.size - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.size - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn selected(&mut self) -> usize {
        match self.state.selected() {
            Some(i) => i,
            None => {
                self.state.select(Some(0));
                0
            }
        }
    }

    pub fn set_selected(&mut self, index: usize) {
        let index = usize::min(self.size - 1, index);
        self.state.select(Some(index));
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ],
    )
    .split(r);

    Layout::new(
        Direction::Horizontal,
        [
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ],
    )
    .split(popup_layout[1])[1]
}

fn list_from_library_items(name: String, items: &Vec<LibraryItem>) -> List {
    let items: Vec<ListItem> = items.iter().map(|item| item.as_list_item()).collect();
    List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Title::from(name).alignment(Alignment::Left)),
        )
        .highlight_style(Style::new().reversed())
        .highlight_symbol(">> ")
}

pub fn render(app: &mut App, f: &mut Frame) {
    let vertical = Layout::new(
        Direction::Vertical,
        [
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(1),
        ],
    )
    .split(f.size());
    let horizontal = Layout::new(
        Direction::Horizontal,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .split(vertical[2]);

    // Get all the stuff we actually want to render, and the information to do so
    let selected = app.get_selected_item();
    //TODO: Add comments to this is less of a mess, or at least a more understandable mess
    let (first, mut first_state, second, mut second_state) = match selected {
        (cat, None) => {
            let index = cat.counter.selected();
            let item = &cat.items[index];
            let (second, state): (&Vec<LibraryItem>, &StatefulListCounter) = match item {
                LibraryItem::Document(_) => (Vec::new(), StatefulListCounter::default()),
                LibraryItem::Category(cat) => (&cat.items, &cat.counter),
            };
            (
                list_from_library_items(cat.name().to_string(), &cat.items),
                cat.counter,
                list_from_library_items(item.name().to_string(), &second),
                state,
            )
        }
        (parent, Some(selected)) => match selected {
            LibraryItem::Document(_) => {
                let index = parent.counter.selected();
                let item = &parent.items[index];
                let (second, state): (&Vec<LibraryItem>, &StatefulListCounter) = match item {
                    LibraryItem::Document(_) => (Vec::new(), StatefulListCounter::default()),
                    LibraryItem::Category(cat) => (&cat.items, &cat.counter),
                };
                (
                    list_from_library_items(parent.name().to_string(), &parent.items),
                    parent.counter,
                    list_from_library_items(item.name().to_string(), &second),
                    state,
                )
            }
            LibraryItem::Category(mut cat) => {
                let index = cat.counter.selected();
                let item = &cat.items[index];
                let (second, state): (&Vec<LibraryItem>, &StatefulListCounter) = match item {
                    LibraryItem::Document(_) => (Vec::new(), StatefulListCounter::default()),
                    LibraryItem::Category(cat) => (&cat.items, &cat.counter),
                };
                (
                    list_from_library_items(cat.name().to_string(), &cat.items),
                    cat.counter,
                    list_from_library_items(item.name().to_string(), &second),
                    state,
                )
            }
        },
    };

    // Render the first list
    f.render_stateful_widget(first, horizontal[0], &mut first_state.state);
    // Generate scrollbar
    let mut scrollbar_state =
        ScrollbarState::new(first_state.size).position(first_state.selected());
    // Render scroll bar
    f.render_stateful_widget(
        Scrollbar::default().orientation(VerticalRight),
        horizontal[0],
        &mut scrollbar_state,
    );

    // Render the second list
    f.render_stateful_widget(second, horizontal[1], &mut second_state.state);
    // Generate scrollbar
    let mut scrollbar_state =
        ScrollbarState::new(second_state.size).position(second_state.selected());
    // Render scroll bar
    f.render_stateful_widget(
        Scrollbar::default().orientation(VerticalRight),
        horizontal[1],
        &mut scrollbar_state,
    );

    // Render the title
    f.render_widget(
        Paragraph::new("Apocalypse Library Download Tool")
            .bold()
            .alignment(Alignment::Center),
        vertical[0],
    );

    // Render the total
    let total = app.category.human_readable_size();
    f.render_widget(
        Paragraph::new(format!("Total Enabled Size: {total}"))
            .bold()
            .alignment(Alignment::Center),
        vertical[3],
    );
}
