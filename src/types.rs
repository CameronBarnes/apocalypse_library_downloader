use std::cmp::Reverse;

use humansize::WINDOWS;
use ratatui::{
    style::{Modifier, Style},
    widgets::ListItem,
};
use serde::Deserialize;

use crate::term::{app::SortStyle, ui::StatefulListCounter};

#[derive(Debug, Deserialize)]
/// Stores either a Category or Document so that Categories may store either
pub enum LibraryItem {
    Document(Document),
    Category(Category),
}

impl LibraryItem {
    /// Returns the size of the `LibraryItem` in bytes
    ///
    ///  # Arguments
    ///
    ///  * `enabled_only`- should it only include the size of enabled items
    ///
    pub fn size(&self, enabled_only: bool) -> u64 {
        match self {
            Self::Document(doc) => {
                if enabled_only {
                    doc.enabled_size()
                } else {
                    doc.size()
                }
            }
            Self::Category(cat) => {
                if enabled_only {
                    cat.enabled_size()
                } else {
                    cat.size(false)
                }
            }
        }
    }

    /// Returns the name of the contained item
    pub fn name(&self) -> &str {
        match self {
            Self::Document(doc) => doc.name(),
            Self::Category(cat) => cat.name(),
        }
    }

    /// Returns the size of the item formatted to be human readable
    pub fn human_readable_size(&self) -> String {
        match self {
            Self::Document(doc) => doc.human_readable_size(),
            Self::Category(cat) => cat.human_readable_size(),
        }
    }

    /// Returns if the item is enabled
    pub const fn enabled(&self) -> bool {
        match self {
            Self::Document(doc) => doc.enabled,
            Self::Category(cat) => cat.enabled,
        }
    }

    /// Set the state of the contained item as enabled or not
    /// Returns the state of the item at the end of the function
    /// If the item cant be downloaded, then it wont be set as enabled
    ///
    /// # Arguments
    ///
    ///  * `enabled` - Should the item be enabled, if it can be downloaded
    ///
    pub fn set_enabled(&mut self, enabled: bool) -> bool {
        match self {
            Self::Document(doc) => {
                if doc.can_download() {
                    doc.enabled = enabled;
                } else {
                    doc.enabled = false;
                }
                doc.enabled
            }
            Self::Category(cat) => {
                if cat.can_download() {
                    cat.enabled = enabled;
                } else {
                    cat.enabled = false;
                }
                cat.enabled
            }
        }
    }

    /// Recursively enable all contained items, if they can be downloaded
    /// Ignores single selection category contents
    pub fn set_enabled_recursive(&mut self) {
        match self {
            Self::Document(doc) => {
                doc.enabled = doc.can_download();
            }
            Self::Category(cat) => {
                cat.enabled = cat.can_download();
                if !cat.single_selection() {
                    cat.items.iter_mut().for_each(Self::set_enabled_recursive);
                }
            }
        }
    }

    /// Returns if the item can be downloaded
    pub fn can_download(&self) -> bool {
        match self {
            Self::Document(doc) => doc.can_download(),
            Self::Category(cat) => cat.can_download(),
        }
    }

    /// Converts the item to a ratatui `ListItem`
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

    /// Returns if the contained item is a Document
    pub const fn is_document(&self) -> bool {
        match self {
            Self::Document(_) => true,
            Self::Category(_) => false,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize, Clone, Copy)]
/// The download method to use for a `Document`
pub enum DownloadType {
    /// Download files with HTTP(s) GET requests
    Http,
    /// Download files by running the rsync application
    Rsync,
    /// Supports either HTTP GET or Rsync. Prefers Rsync by default
    Either,
}

#[derive(Debug, Deserialize)]
/// Represents a File or Group of files to download
pub struct Document {
    /// The name of the Document(s)
    name: String,
    /// The path of the File(s) to get
    url: String,
    /// The total size of the File(s) in bytes
    size: u64,
    /// The method to use to download the File(s)
    download_type: DownloadType,
    /// Should these File(s) be downloaded
    pub enabled: bool,
}

impl Document {
    #[allow(unused)]
    // Probably doesnt need to be here, as we dont actually use this in this executable
    pub fn new(name: String, url: String, size: u64, d_type: DownloadType) -> Self {
        let enabled = d_type != DownloadType::Rsync || (!crate::IS_WINDOWS && *crate::HAS_RSYNC);
        Self {
            name,
            url,
            size,
            download_type: d_type,
            enabled,
        }
    }

    /// Returns a reference to the name of this Document
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns a reference to the url where the File(s) can be found
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Returns the size in bytes
    pub const fn size(&self) -> u64 {
        self.size
    }

    /// Returns the download type of this Document
    pub const fn download_type(&self) -> DownloadType {
        self.download_type
    }

    /// Returns the size of this document, or zero if it's not enabled
    pub const fn enabled_size(&self) -> u64 {
        if self.enabled {
            self.size
        } else {
            0
        }
    }

    /// Returns if we can download this Document
    /// In cases such as a rsync Document on a windows system we cant download it
    pub fn can_download(&self) -> bool {
        self.download_type != DownloadType::Rsync || (!crate::IS_WINDOWS && *crate::HAS_RSYNC)
    }

    /// Returns the size of the item formatted to be human readable
    pub fn human_readable_size(&self) -> String {
        humansize::format_size(self.size, WINDOWS)
    }
}

#[derive(Debug, Deserialize)]
/// Contains a navigable list of items grouped together. Items may be either Documents or
/// Categories of their own
pub struct Category {
    /// The name of the category
    name: String,
    /// The items contained in the category, may be a mix of Documents and Categories
    pub items: Vec<LibraryItem>,
    /// Can only one item be selected at a time
    single_selection: bool,
    /// Is this category enabled for download
    pub enabled: bool,
    #[serde(skip)]
    /// The list cursor pointer
    pub counter: StatefulListCounter,
}

impl Category {
    /// Creates a new Category with the provided items and settings
    pub fn new(name: String, mut items: Vec<LibraryItem>, single_selection: bool) -> Self {
        if single_selection {
            // Only one option can be enabled at a time with single selection
            (1..items.len()).for_each(|i| {
                items[i].set_enabled(false);
            });
        }
        let enabled = items.iter().any(LibraryItem::can_download);
        let len = items.len();
        Self {
            name,
            items,
            enabled,
            single_selection,
            counter: StatefulListCounter::new(len),
        }
    }

    /// Fixes the `StatefulListCounter` which is broken when this object is deserialized
    pub fn fix_counter(&mut self) {
        self.counter = StatefulListCounter::new(self.items.len());
        self.counter.selected();
        for item in &mut self.items {
            match item {
                LibraryItem::Document(_) => {}
                LibraryItem::Category(cat) => cat.fix_counter(),
            }
        }
    }

    /// Returns a reference to the Category's name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the size of the `Category` and contained items in bytes
    ///
    ///  # Arguments
    ///
    ///  * `enabled_only`- should it only include the size of enabled items
    ///
    pub fn size(&self, enabled_only: bool) -> u64 {
        self.items.iter().map(|item| item.size(enabled_only)).sum()
    }

    /// Returns the size of all enabled items, or zero if this `Category` is disabled
    pub fn enabled_size(&self) -> u64 {
        if self.enabled {
            self.size(true)
        } else {
            0
        }
    }

    /// Returns if any `LibraryItem` contained in this `Category` can be downloaded
    pub fn can_download(&self) -> bool {
        self.items.iter().any(LibraryItem::can_download)
    }

    /// Returns if only a single item in this `Category` may be enabled at a time
    pub const fn single_selection(&self) -> bool {
        self.single_selection
    }

    /// Returns the enabled size of the `Category` formatted to be human readable
    pub fn human_readable_size(&self) -> String {
        humansize::format_size(self.size(true), WINDOWS)
    }

    /// Proceeds down the specified depth to find the last selected `Category`
    pub fn get_selected_category(&mut self, depth: usize) -> (&mut Self, usize) {
        if depth == 0 || self.is_selected_last() {
            (self, depth)
        } else if self.is_selected_category() {
            match &mut self.items[self.counter.selected()] {
                LibraryItem::Document(_) => unreachable!(),
                LibraryItem::Category(cat) => cat.get_selected_category(depth - 1),
            }
        } else {
            (self, depth + 1)
        }
    }

    /// Returns if the item currently selected in this `Category` is a `Category`
    pub fn is_selected_category(&self) -> bool {
        let index = self.counter.clone().selected();
        match &self.items[index] {
            LibraryItem::Document(_) => false,
            LibraryItem::Category(_) => true,
        }
    }

    /// Returns if the currently selected item is a `Category` and that `Category` only contains
    /// `Documents`
    pub fn is_selected_last(&self) -> bool {
        let index = self.counter.clone().selected();
        match &self.items[index] {
            LibraryItem::Document(_) => false,
            LibraryItem::Category(cat) => cat.items.iter().all(LibraryItem::is_document),
        }
    }

    /// If this `Category` is not `single_selection` toggle the state of all items in this `Category`
    pub fn toggle_all_items(&mut self) {
        if !self.single_selection() {
            self.items.iter_mut().for_each(|item| {
                let state = !item.enabled();
                item.set_enabled(state);
            });
        }
    }

    /// Toggles the currently selected item in the `Category`
    pub fn toggle_selected_item(&mut self) {
        let single_selection = self.single_selection();
        let index = self.counter.selected();
        let item = &self.items[index];
        let (enabled, can_download) = (item.enabled(), item.can_download());
        if single_selection {
            // Only one item can be enabled at a time
            if !enabled && can_download {
                // Item can be enabled, do so and disable
                // all other items
                self.items.iter_mut().for_each(|item| {
                    item.set_enabled(false);
                });
                self.items[index].set_enabled(true);
            } else if item.enabled() {
                // Item is enabled, disable it and enable the first
                // downloadable item
                self.items[index].set_enabled(false);
                for item in &mut self.items {
                    if item.can_download() {
                        item.set_enabled(true);
                        break;
                    }
                }
            }
        } else {
            // Not single selection, just toggle the item
            self.items[index].set_enabled(!enabled);
        }
    }

    /// Sorts the `Category` contents by the provided style
    ///
    ///  # Arguments
    ///
    ///  * `style` - the sort order to use
    ///  Currently either Alphabetically A-Z or by size decending
    ///
    pub fn sort(&mut self, style: SortStyle) {
        let name = self.items[self.counter.selected()].name().to_owned();
        let old_selected = self.counter.selected();
        match style {
            SortStyle::Alphabetical => {
                self.items
                    .sort_unstable_by_key(|item| item.name().to_string());
            }
            SortStyle::Size => {
                self.items.sort_unstable_by_key(|item| {
                    let size = match item {
                        LibraryItem::Document(doc) => doc.size(),
                        LibraryItem::Category(cat) => cat.size(true),
                    };
                    Reverse(size)
                });
            }
        }
        if old_selected != 0 {
            let found = self.items.iter().enumerate().find(|enumerated| enumerated.1.name().eq_ignore_ascii_case(&name));
            if let Some((i, _)) = found {
                self.counter.set_selected(i);
            }
        }
    }

    /// Adds the provided `LibraryItem` to this `Category`
    /// If the provided item is a `Category` it will check to see if it has the same name as an
    /// existing `Category` and merge them together
    pub fn add(&mut self, mut item: LibraryItem) {
        if self.single_selection && !self.items.is_empty() {
            item.set_enabled(false);
        }
        match item {
            LibraryItem::Document(_) => self.items.push(item),
            LibraryItem::Category(category) => {
                if category.items.is_empty() {
                    return;
                }
                if let Some(merge) = self.items.iter_mut().find_map(|e| match e {
                    LibraryItem::Document(_) => None,
                    LibraryItem::Category(cat) => {
                        if cat.name.eq_ignore_ascii_case(category.name()) {
                            Some(cat)
                        } else {
                            None
                        }
                    }
                }) {
                    // End of condition, merge the two categories if their names match
                    for item in category.items {
                        merge.add(item);
                    }
                } else {
                    self.items.push(LibraryItem::Category(category));
                }
            }
        }
    }
}
