/// Conversion-only semantic model. Space persistence and asset authorization stay in Spaces.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DocumentSource {
    pub title: String,
    pub markdown: String,
    pub citations: Vec<Citation>,
    pub images: Vec<ImageSource>,
    #[serde(default)]
    pub rendered_images: Vec<RenderedImage>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ImageSource {
    pub destination: String,
    pub file: super::input::DocumentFileSource,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RenderedImage {
    pub destination: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NoteSource {
    pub title: String,
    pub content: serde_json::Value,
    pub citations: Vec<Citation>,
    pub images: Vec<ImageSource>,
    #[serde(default)]
    pub rendered_images: Vec<RenderedImage>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Citation {
    pub title: String,
    pub quote: String,
}

#[derive(Debug, PartialEq)]
pub(crate) struct Document {
    pub title: String,
    pub blocks: Vec<Block>,
    pub leading_title_removed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CalloutKind {
    Note,
    Tip,
    Important,
    Warning,
    Caution,
}

impl CalloutKind {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Note => "NOTE",
            Self::Tip => "TIP",
            Self::Important => "IMPORTANT",
            Self::Warning => "WARNING",
            Self::Caution => "CAUTION",
        }
    }

    pub(crate) const fn css_name(self) -> &'static str {
        match self {
            Self::Note => "note",
            Self::Tip => "tip",
            Self::Important => "important",
            Self::Warning => "warning",
            Self::Caution => "caution",
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum Block {
    Paragraph(Vec<Inline>),
    Heading {
        level: u8,
        content: Vec<Inline>,
    },
    Quote(Vec<Block>),
    Callout {
        kind: CalloutKind,
        blocks: Vec<Block>,
    },
    Code {
        language: String,
        text: String,
    },
    List {
        start: Option<u64>,
        items: Vec<Vec<Block>>,
    },
    Table(Table),
    Math(String),
    Rule,
}

#[derive(Debug, PartialEq)]
pub(crate) struct Table {
    pub alignments: Vec<Alignment>,
    pub header: Vec<Vec<Inline>>,
    pub rows: Vec<Vec<Vec<Inline>>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Alignment {
    Left,
    Center,
    Right,
}

#[derive(Debug, PartialEq)]
pub(crate) enum Inline {
    Text(String),
    Strong(Vec<Inline>),
    Emphasis(Vec<Inline>),
    Strike(Vec<Inline>),
    Code(String),
    Math(String),
    DisplayMath(String),
    Link {
        url: String,
        content: Vec<Inline>,
    },
    Image {
        destination: String,
        alt: Vec<Inline>,
    },
    Task(bool),
    SoftBreak,
    HardBreak,
}
