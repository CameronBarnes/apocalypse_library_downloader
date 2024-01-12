use humansize::WINDOWS;

#[derive(Debug)]
pub enum LibraryItem {
    Document(Document),
    Category(Category),
}

impl LibraryItem {
    pub fn size(&self, enabled_only: bool) -> usize {
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
}

#[derive(Debug, PartialEq, Eq)]
pub enum DownloadType {
    Http,
    Rsync,
    Either,
}

#[derive(Debug)]
pub struct Document {
    name: String,
    url: String,
    size: usize,
    download_type: DownloadType,
    pub enabled: bool,
}

impl Document {
    pub fn new(name: String, url: String, size: usize, d_type: DownloadType) -> Self {
        let enabled = d_type != DownloadType::Rsync || !crate::IS_WINDOWS;
        Document{name, url, size, download_type: d_type, enabled}
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn download_type(&self) -> &DownloadType {
        &self.download_type
    }

    pub fn enabled_size(&self) -> usize {
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

#[derive(Debug)]
pub struct Category {
    name: String,
    pub items: Vec<LibraryItem>,
    single_selection: bool,
    pub enabled: bool,
}

impl Category {
    pub fn new(name: String, mut items: Vec<LibraryItem>, single_selection: bool) -> Self {
        if single_selection { // Only one option can be enabled at a time with single selection
            (1..items.len()).for_each(|i| {
                items[i].set_enabled(false);
            });
        }
        let enabled = items.iter().any(LibraryItem::can_download);
        Category{name, items, enabled, single_selection}
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn size(&self, enabled_only: bool) -> usize {
        self.items.iter().map(|item| item.size(enabled_only)).sum()
    }

    pub fn enabled_size(&self) -> usize {
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
}
