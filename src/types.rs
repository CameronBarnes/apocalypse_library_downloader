use humansize::WINDOWS;
use ratatui::{widgets::ListItem, style::{Style, Modifier}};
use serde::Deserialize;

use crate::term::ui::StatefulListCounter;

#[derive(Debug, Deserialize)]
pub enum LibraryItem {
    Document(Document),
    Category(Category),
}

impl LibraryItem {
    pub fn size(&self, enabled_only: bool) -> u64 {
        match self {
            LibraryItem::Document(doc) => {
                if enabled_only {
                    doc.enabled_size()
                } else {
                    doc.size()
                }
            },
            LibraryItem::Category(cat) => {
                if enabled_only {
                    cat.enabled_size()
                } else {
                    cat.size(false)
                }
            },
        }
    }

    pub fn name(&self) -> &str {
        match self {
            LibraryItem::Document(doc) => doc.name(),
            LibraryItem::Category(cat) => cat.name(),
        }
    }

    pub fn human_readable_size(&self) -> String {
        match self {
            LibraryItem::Document(doc) => doc.human_readable_size(),
            LibraryItem::Category(cat) => cat.human_readable_size(),
        }
    }

    pub fn enabled(&self) -> bool {
        match self {
            LibraryItem::Document(doc) => doc.enabled,
            LibraryItem::Category(cat) => cat.enabled,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) -> bool {
        match self {
            LibraryItem::Document(doc) => {
                if doc.can_download() {
                    doc.enabled = enabled;
                } else {
                    doc.enabled = false;
                }
                doc.enabled
            },
            LibraryItem::Category(cat) => {
                if cat.can_download() {
                    cat.enabled = enabled;
                } else {
                    cat.enabled = false;
                }
                cat.enabled
            },
        }
    }

    pub fn can_download(&self) -> bool {
        match self {
            LibraryItem::Document(doc) => doc.can_download(),
            LibraryItem::Category(cat) => cat.can_download(),
        }
    }

    pub fn as_list_item(&self) -> ListItem {

        let name = self.name();
        let size = self.human_readable_size();
        let item = ListItem::new(format!("{name}:  {size}"));
        let mut style = Style::default();
        if !self.enabled() {
            style = style.add_modifier(Modifier::DIM);
        }
        if !self.can_download() {
            style = style.add_modifier(Modifier::CROSSED_OUT);
        }
        item.style(style)

    }
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub enum DownloadType {
    Http,
    Rsync,
    Either,
}

#[derive(Debug, Deserialize)]
pub struct Document {
    name: String,
    url: String,
    size: u64,
    download_type: DownloadType,
    pub enabled: bool,
}

impl Document {
    pub fn new(name: String, url: String, size: u64, d_type: DownloadType) -> Self {
        let enabled = d_type != DownloadType::Rsync || !crate::IS_WINDOWS;
        Document{name, url, size, download_type: d_type, enabled}
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn download_type(&self) -> &DownloadType {
        &self.download_type
    }

    pub fn enabled_size(&self) -> u64 {
        if self.enabled {
            self.size
        } else {
            0
        }
    }

    /// In cases such as a rsync Document on a windows system we cant download it
    pub fn can_download(&self) -> bool {
        self.download_type != DownloadType::Rsync || !crate::IS_WINDOWS
    }

    pub fn human_readable_size(&self) -> String {
        humansize::format_size(self.size, WINDOWS)
    }
}

#[derive(Debug, Deserialize)]
pub struct Category {
    name: String,
    pub items: Vec<LibraryItem>,
    single_selection: bool,
    pub enabled: bool,
    #[serde(skip)]
    pub counter: StatefulListCounter,
}

impl Category {
    pub fn new(name: String, mut items: Vec<LibraryItem>, single_selection: bool) -> Self {
        if single_selection { // Only one option can be enabled at a time with single selection
            (1..items.len()).for_each(|i| {
                items[i].set_enabled(false);
            });
        }
        let enabled = items.iter().any(LibraryItem::can_download);
        let len =  items.len();
        Category{name, items, enabled, single_selection, counter: StatefulListCounter::new(len)}
    }

    pub fn fix_counter(&mut self) {
        self.counter = StatefulListCounter::new(self.items.len());
        for item in &mut self.items {
            match item {
                LibraryItem::Document(_) => {},
                LibraryItem::Category(cat) => cat.fix_counter(),
            }
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn size(&self, enabled_only: bool) -> u64 {
        self.items.iter().map(|item| item.size(enabled_only)).sum()
    }

    pub fn enabled_size(&self) -> u64 {
        if self.enabled {
            self.size(true)
        } else {
            0
        }
    }

    pub fn can_download(&self) -> bool {
        self.items.iter().any(|item| item.can_download())
    }

    pub fn single_selection(&self) -> bool {
        self.single_selection
    }

    pub fn human_readable_size(&self) -> String {
        humansize::format_size(self.size(true), WINDOWS)
    }

    pub fn get_selected_category(&mut self, depth: usize) -> (&mut Category, usize) {

        if depth == 0 {
            panic!("Shouldnt ever happen")
        } else if depth == 1 {
            (self, 1)
        } else {
            let index = self.counter.selected();
            let result = self.items[index];
            match result {
                LibraryItem::Document(_) => (self, depth),
                LibraryItem::Category(mut cat) => cat.get_selected_category(depth - 1),
            }
        }

    }
}
