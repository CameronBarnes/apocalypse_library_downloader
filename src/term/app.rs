use crate::types::{Category, LibraryItem};

#[derive(Debug, Clone, Copy)]
pub enum SortStyle {
    Alphabetical,
    Size,
}

#[derive(Debug)]
pub struct App {
    pub should_quit: bool,
    pub category: Category,
    pub depth: usize,
    pub download: bool,
    sort_style: SortStyle,
}

impl App {
    pub fn new(mut category: Category) -> Self {
        category.sort(SortStyle::Alphabetical);
        category.items.iter_mut().for_each(|item| {
            match item {
                LibraryItem::Document(_) => {},
                LibraryItem::Category(cat) => cat.sort(SortStyle::Alphabetical),
            }
        });
        App {
            should_quit: false,
            category,
            depth: 0,
            download: false,
            sort_style: SortStyle::Alphabetical
        }
    }

    pub fn sort(&mut self) {
        let style = self.sort_style;
        let (cat, _) = self.get_selected_category();
        cat.sort(style);
        let index = cat.counter.selected();
        match &mut cat.items[index] {
            LibraryItem::Document(_) => {},
            LibraryItem::Category(cat) => cat.sort(style),
        }
    }

    pub fn toggle_sort_style(&mut self) {
        match self.sort_style {
            SortStyle::Alphabetical => self.sort_style = SortStyle::Size,
            SortStyle::Size => self.sort_style = SortStyle::Alphabetical,
        }
        self.sort();
    }

    pub fn get_selected_category(&mut self) -> (&mut Category, usize) {
        self.category.get_selected_category(self.depth)
    }

    pub fn left(&mut self) {
        self.depth = self.depth.saturating_sub(1);
        self.sort();
    }

    pub fn right(&mut self) {
        self.depth += 1;
        let (_, depth) = self.get_selected_category();
        self.depth -= depth.saturating_sub(1);
        self.sort();
    }

    pub fn next(&mut self) {
        let (cat, depth) = self.get_selected_category();
        if !cat.is_selected_last() || depth == 0 {
            cat.counter.next();
        } else {
            let index = cat.counter.selected();
            match &mut cat.items[index] {
                LibraryItem::Document(_) => unreachable!(),
                LibraryItem::Category(cat) => cat.counter.next(),
            }
        }
        self.sort();
    }

    pub fn previous(&mut self) {
        let (cat, depth) = self.get_selected_category();
        if !cat.is_selected_last() || depth == 0 {
            cat.counter.previous();
        } else {
            let index = cat.counter.selected();
            match &mut cat.items[index] {
                LibraryItem::Document(_) => unreachable!(),
                LibraryItem::Category(cat) => cat.counter.previous(),
            }
        }
        self.sort();
    }

    pub fn home(&mut self) {
        let (cat, depth) = self.get_selected_category();
        if !cat.is_selected_last() || depth == 0 {
            cat.counter.set_selected(0);
        } else {
            let index = cat.counter.selected();
            match &mut cat.items[index] {
                LibraryItem::Document(_) => unreachable!(),
                LibraryItem::Category(cat) => cat.counter.set_selected(0),
            }
        }
    }

    pub fn end(&mut self) {
        let (cat, depth) = self.get_selected_category();
        if !cat.is_selected_last() || depth == 0 {
            let max = cat.counter.size();
            cat.counter.set_selected(max - 1);
        } else {
            let index = cat.counter.selected();
            match &mut cat.items[index] {
                LibraryItem::Document(_) => unreachable!(),
                LibraryItem::Category(cat) => {
                    let max = cat.counter.size();
                    cat.counter.set_selected(max - 1);
                }
            }
        }
    }

    pub fn toggle(&mut self) {
        let result = self.get_selected_category();
        match result {
            (cat, 0) => cat.toggle_selected_item(),
            (cat, _) => {
                let index = cat.counter.selected();
                match &mut cat.items[index] {
                    LibraryItem::Document(_) => unreachable!(),
                    LibraryItem::Category(cat) => cat.toggle_selected_item(),
                }
            }
        }
    }

    pub fn toggle_all(&mut self) {
        let result = self.get_selected_category();
        match result {
            (cat, 0) => cat.toggle_all_items(),
            (cat, _) => {
                let index = cat.counter.selected();
                match &mut cat.items[index] {
                    LibraryItem::Document(_) => unreachable!(),
                    LibraryItem::Category(cat) => cat.toggle_all_items(),
                }
            }
        }
    }

    pub fn tick(&self) {}

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
