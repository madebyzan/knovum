mod docx;
mod markdown;
mod pdf;
mod pptx;
mod text;

use super::document::DocumentSource;

fn source(markdown: &str) -> DocumentSource {
    DocumentSource {
        markdown: markdown.into(),
        ..DocumentSource::default()
    }
}

fn fixture() -> DocumentSource {
    DocumentSource {
        title: "中文 English document".into(),
        markdown: include_str!("common.md").into(),
        ..DocumentSource::default()
    }
}
