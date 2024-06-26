use std::{sync::RwLock, rc::Rc};

use ratatui::{
    prelude::*,
    widgets::{
        block::Title, Block, Borders, Clear, List, ListItem, ListState, Padding, Paragraph,
        Scrollbar, ScrollbarOrientation::VerticalRight, ScrollbarState, Wrap,
    },
};

use crate::types::{Category, LibraryItem};

use super::app::App;

#[derive(Debug, Default, Clone)]
pub struct StatefulListCounter {
    state: Rc<RwLock<ListState>>,
    size: usize,
}

impl StatefulListCounter {
    pub fn new(size: usize) -> Self {
        Self {
            state: Rc::new(RwLock::new(ListState::default())),
            size,
        }
    }

    pub fn next(&mut self) {
        let selected = self.state.try_write().unwrap().selected();
        let i = match selected {
            Some(i) => {
                if i >= self.size - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.try_write().unwrap().select(Some(i));
    }

    pub fn previous(&mut self) {
        let selected = self.state.try_write().unwrap().selected();
        let i = match selected {
            Some(i) => {
                if i == 0 {
                    self.size - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.try_write().unwrap().select(Some(i));
    }

    pub const fn size(&self) -> usize {
        self.size
    }

    pub fn selected(&mut self) -> usize {
        let selected = self.state.try_write().unwrap().selected();
        if let Some(i) = selected {
            i
        } else {
            self.state.try_write().unwrap().select(Some(0));
            0
        }
    }

    pub fn set_selected(&mut self, index: usize) {
        let index = usize::min(self.size - 1, index);
        self.state.try_write().unwrap().select(Some(index));
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

fn list_from_library_items(name: String, items: Option<&Vec<LibraryItem>>, selected: bool) -> List {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Title::from(name).alignment(Alignment::Left));
    let mut highlight_style = Style::new().reversed();
    if selected {
        highlight_style = highlight_style.light_cyan();
    }

    if let Some(items) = items {
        let items: Vec<ListItem> = items.iter().map(|item| item.as_list_item()).collect();
        List::new(items)
            .block(block)
            // Contributed this feature upstream just so that I could use it here
            .scroll_padding(1)
            .highlight_style(highlight_style)
            .highlight_symbol(">> ")
    } else {
        let empty: Vec<ListItem> = Vec::new();
        List::new(empty)
            .block(block.dim())
            // Contributed this feature upstream just so that I could use it here
            .scroll_padding(1)
            .highlight_style(highlight_style)
            .highlight_symbol(">> ")
    }
}

fn get_list_from_category_selected(
    category: &Category,
    selected: bool,
) -> (List, StatefulListCounter) {
    if category.items.is_empty() {
        return (
            list_from_library_items(category.name().to_string(), None, selected),
            StatefulListCounter::default(),
        );
    }
    let index = category.counter.clone().selected();
    let item = &category.items[index];
    match item {
        LibraryItem::Document(_) => (
            list_from_library_items(category.name().to_string(), None, selected),
            StatefulListCounter::default(),
        ),
        LibraryItem::Category(cat) => (
            list_from_library_items(cat.name().to_string(), Some(&cat.items), selected),
            cat.counter.clone(),
        ),
    }
}

fn get_lists_from_app(app: &mut App) -> (List, StatefulListCounter, List, StatefulListCounter) {
    let result = app.get_selected_category();
    match result {
        (cat, 0) => {
            let state = cat.counter.clone();
            let first = list_from_library_items(cat.name().to_string(), Some(&cat.items), true);
            let (second, second_state) = get_list_from_category_selected(&*cat, false);
            (first, state, second, second_state)
        }
        (parent, _) => {
            let index = parent.counter.selected();
            let item = &parent.items[index];
            if parent.is_selected_last() {
                match item {
                    LibraryItem::Document(_) => unreachable!(),
                    LibraryItem::Category(_) => {
                        let state = parent.counter.clone();
                        let name = parent.name().to_string();
                        let first = list_from_library_items(name, Some(&parent.items), false);
                        let (second, second_state) =
                            get_list_from_category_selected(&*parent, true);
                        (first, state, second, second_state)
                    }
                }
            } else {
                match item {
                    LibraryItem::Document(_) => {
                        let state = parent.counter.clone();
                        let name = parent.name().to_string();
                        let first = list_from_library_items(name, Some(&parent.items), true);
                        let (second, second_state) =
                            get_list_from_category_selected(&*parent, false);
                        (first, state, second, second_state)
                    }
                    LibraryItem::Category(cat) => {
                        let state = cat.counter.clone();
                        let (second, second_state) = get_list_from_category_selected(cat, true);
                        (
                            list_from_library_items(
                                cat.name().to_string(),
                                Some(&cat.items),
                                false,
                            ),
                            state,
                            second,
                            second_state,
                        )
                    }
                }
            }
        }
    }
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
    let (first, mut first_state, second, mut second_state) = get_lists_from_app(app);

    // Render the first list
    f.render_stateful_widget(first, horizontal[0], &mut first_state.state.try_write().unwrap());
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
    f.render_stateful_widget(second, horizontal[1], &mut second_state.state.try_write().unwrap());
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

    // Render help
    f.render_widget(
        Paragraph::new("ESC or ctrl-C to quit | arrow keys for navigation | space to toggle item | ENTER to download | TAB to toggle everything in the current category | 'S' to change sort mode")
            .bold()
            .alignment(Alignment::Center),
        vertical[1],
    );

    // Render the total
    let total = app.category.human_readable_size();
    f.render_widget(
        Paragraph::new(format!("Total Enabled Size: {total}"))
            .bold()
            .alignment(Alignment::Center),
        vertical[3],
    );

    if app.download {
        let area = centered_rect(60, 60, f.size());
        f.render_widget(Clear, area); // Clear the area so we can render over it
        let paragraph = Paragraph::new(format!(
            "{total}\n\nPress ESC or ctrl-C to go back\nENTER to download files now"
        ))
        .bold()
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Download")
                .title_alignment(Alignment::Center)
                .title_style(Style::default().bold())
                .padding(Padding::new(5, 10, 1, 2)),
        );

        // Render
        f.render_widget(paragraph, area);
    }
}
