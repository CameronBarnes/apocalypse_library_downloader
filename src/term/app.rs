use crate::types::{Category, LibraryItem};

#[derive(Debug)]
pub struct App {
    pub should_quit: bool,
    pub category: Category,
    pub depth: usize,
    pub download: bool,
}

impl App {
    pub fn new(mut categories: Vec<LibraryItem>) -> Self {
        categories
            .iter_mut()
            .flat_map(|item| match item {
                LibraryItem::Document(_) => None,
                LibraryItem::Category(cat) => Some(cat),
            })
            .for_each(|cat| cat.fix_counter());
        let mut category = Category::new(String::from("Apocalypse Library"), categories, false);
        category.fix_counter();
        App {
            should_quit: false,
            category,
            depth: 0,
            download: false,
        }
    }

    pub fn get_selected_category(&mut self) -> (&mut Category, usize) {
        self.category.get_selected_category(self.depth)
    }

    pub fn left(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    pub fn right(&mut self) {
        self.depth += 1;
        let (_, depth) = self.get_selected_category();
        self.depth -= depth.saturating_sub(1);
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
                },
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

    pub fn tick(&self) {}

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
