mod blocks;
mod containers;
mod inlines;

use pulldown_cmark::{Event, Options, Parser};

use super::document::{Block, Document, DocumentSource, Inline};

const MAX_SOURCE_BYTES: usize = 4 * 1024 * 1024;
const MAX_EVENTS: usize = 200_000;
const MAX_DEPTH: usize = 64;

pub(crate) fn parse(source: &DocumentSource) -> Result<Document, String> {
    validate_source(source)?;
    let options = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
    let mut reader = Reader {
        parser: Parser::new_ext(&source.markdown, options),
        events: 0,
    };
    let mut blocks = reader.blocks(None, 0)?;
    let title = source.title.trim().to_owned();
    if let Some(Block::Heading { level: 1, content }) = blocks.first() {
        if !title.is_empty() && super::text::inline_text(content, false).trim() == title {
            blocks.remove(0);
        }
    }
    if !source.citations.is_empty() {
        blocks.push(Block::Heading {
            level: 2,
            content: vec![Inline::Text("Sources".into())],
        });
        for (index, citation) in source.citations.iter().enumerate() {
            blocks.push(Block::Paragraph(vec![
                Inline::Text(format!("[C{}] ", index + 1)),
                Inline::Strong(vec![Inline::Text(citation.title.clone())]),
                Inline::Text(format!(" — {}", citation.quote)),
            ]));
        }
    }
    Ok(Document { title, blocks })
}

fn validate_source(source: &DocumentSource) -> Result<(), String> {
    if source.citations.len() > 4096 {
        return Err("Document export exceeds 4096 citations".into());
    }
    let mut total = 0usize;
    for value in std::iter::once(source.title.as_str())
        .chain(std::iter::once(source.markdown.as_str()))
        .chain(source.citations.iter().flat_map(|citation| {
            [citation.title.as_str(), citation.quote.as_str()]
        }))
    {
        total = total.saturating_add(value.len());
        if total > MAX_SOURCE_BYTES {
            return Err("Document export input exceeds 4 MiB".into());
        }
        if value.chars().any(|c| {
            (c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
                || matches!(c, '\u{fffe}' | '\u{ffff}')
        }) {
            return Err("Document export input contains unsupported control characters".into());
        }
    }
    Ok(())
}

struct Reader<'a> {
    parser: Parser<'a>,
    events: usize,
}

impl<'a> Reader<'a> {
    fn next(&mut self) -> Result<Option<Event<'a>>, String> {
        let event = self.parser.next();
        self.events += usize::from(event.is_some());
        if self.events > MAX_EVENTS {
            return Err("Document export exceeds the Markdown event budget".into());
        }
        Ok(event)
    }

    fn check_depth(depth: usize) -> Result<(), String> {
        if depth > MAX_DEPTH {
            return Err("Document export nesting exceeds 64 levels".into());
        }
        Ok(())
    }
}
