/// Export-only semantic model. Space persistence and asset authorization stay in Spaces.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DocumentSource {
    pub title: String,
    pub markdown: String,
    pub citations: Vec<Citation>,
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
}

#[derive(Debug, PartialEq)]
pub(crate) enum Block {
    Paragraph(Vec<Inline>),
    Heading {
        level: u8,
        content: Vec<Inline>,
    },
    Quote(Vec<Block>),
    Code {
        language: String,
        text: String,
    },
    List {
        start: Option<u64>,
        items: Vec<Vec<Block>>,
    },
    Table(Table),
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
    Link { url: String, content: Vec<Inline> },
    Image { alt: Vec<Inline> },
    Task(bool),
    SoftBreak,
    HardBreak,
}
