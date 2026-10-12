mod content;
mod images;
mod output;
mod validation;

use serde_json::Value;

use crate::documents::generation::document::NoteSource;

fn source(content: Value) -> NoteSource {
    NoteSource {
        title: String::new(),
        content,
        citations: Vec::new(),
        images: Vec::new(),
    }
}
