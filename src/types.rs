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
}

#[derive(Debug)]
pub struct Document {
    name: String,
    url: String,
    size: usize,
    rsync: bool,
    pub enabled: bool,
}

impl Document {
    pub fn new(name: String, url: String, size: usize, rsync: bool) -> Self {
        Document{name, url, size, rsync, enabled: true}
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

    pub fn rsync(&self) -> bool {
        self.rsync
    }

    pub fn enabled_size(&self) -> usize {
        if self.enabled {
            self.size
        } else {
            0
        }
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
    pub fn new(name: String, items: Vec<LibraryItem>, single_selection: bool) -> Self {
        Category{name, items, enabled: true, single_selection}
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

    pub fn single_selection(&self) -> bool {
        self.single_selection
    }

    pub fn human_readable_size(&self) -> String {
        humansize::format_size(self.size(true), WINDOWS)
    }
}
