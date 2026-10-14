mod docx;
mod file_conversion;
mod html;
mod images;
mod links;
mod markdown;
mod pdf;
mod pdf_conversion;
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
        markdown: include_str!("markdown_syntax.md").into(),
        ..DocumentSource::default()
    }
}
