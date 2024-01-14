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
        let result = if self.depth == 0 {
            (&mut self.category, 0)
        } else {
            self.category.get_selected_category(self.depth)
        };
        match result {
            (_, 0) | (_, 1) => {}
            (_, num) => self.depth = num - 1,
        }
        result
    }

    pub fn left(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    pub fn right(&mut self) {
        self.depth += 1;
    }

    pub fn next(&mut self) {
        let (cat, _) = self.get_selected_category();
        cat.counter.next();
    }

    pub fn previous(&mut self) {
        let (cat, _) = self.get_selected_category();
        cat.counter.previous();
    }

    pub fn home(&mut self) {
        let (cat, _) = self.get_selected_category();
        cat.counter.set_selected(0);
    }

    pub fn end(&mut self) {
        let (cat, _) = self.get_selected_category();
        let last = cat.counter.size();
        cat.counter.set_selected(last);
    }

    pub fn toggle(&mut self) {
        let result = self.get_selected_category();
        match result {
            (cat, 0) => cat.enabled = !cat.enabled,
            (cat, _) => {
                cat.toggle_selected_item();
            }
        }
    }

    pub fn tick(&self) {}

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
