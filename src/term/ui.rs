use humansize::WINDOWS;
use ratatui::{prelude::*, widgets::{ListState, ListItem, List, Block, Borders, Paragraph, ScrollbarState, Scrollbar, ScrollbarOrientation::VerticalRight, Padding, Clear, Wrap, block::Title}};

use super::app::App;

#[derive(Debug, Default)]
pub struct StatefulListCounter {
    state: ListState,
    size: usize,
}

impl StatefulListCounter {
    pub fn new(size: usize) -> Self {
        StatefulListCounter{state: ListState::default(), size}
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
    let popup_layout = Layout::new(Direction::Vertical, [
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::new(Direction::Horizontal, [
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}

pub fn render(app: &mut App, f: &mut Frame) {

}
