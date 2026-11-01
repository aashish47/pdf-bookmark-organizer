#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Front,
    Content,
    Back,
}

impl Section {
    pub fn label(self) -> &'static str {
        match self {
            Self::Front => "FRONT",
            Self::Content => "CONTENT",
            Self::Back => "BACK",
        }
    }

    pub fn structural_title(self) -> &'static str {
        match self {
            Self::Front => "Front Matter",
            Self::Content => "Content",
            Self::Back => "Back Matter",
        }
    }
}

/// A flattened bookmark read from the source PDF.
#[derive(Debug, Clone)]
pub struct Bookmark {
    /// Position in the flat outline, starting at zero.
    pub index: usize,
    /// Original outline level, starting at one.
    pub level: usize,
    pub title: String,
    /// 1-based PDF page number.
    pub page: u32,
    pub section: Option<Section>,
}

/// A bookmark in the proposed replacement outline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlineEntry {
    pub level: usize,
    pub title: String,
    /// 1-based PDF page number.
    pub page: u32,
}

pub fn normalize_title(title: &str) -> String {
    title.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn normalized_lower(title: &str) -> String {
    normalize_title(title).to_lowercase()
}
