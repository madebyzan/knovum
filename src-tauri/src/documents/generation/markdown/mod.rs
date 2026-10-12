mod blocks;
mod containers;
mod inlines;
pub(crate) mod ir;
mod ir_inlines;

use pulldown_cmark::{Event, Options, Parser};

use super::document::{Block, Document, DocumentSource, Inline};

pub(super) const MAX_SOURCE_BYTES: usize = 4 * 1024 * 1024;
pub(super) const MAX_EVENTS: usize = 200_000;
pub(super) const MAX_DEPTH: usize = 64;
pub(super) const OPTIONS: Options = Options::ENABLE_TABLES
    .union(Options::ENABLE_STRIKETHROUGH)
    .union(Options::ENABLE_TASKLISTS);

pub(crate) fn parse(source: &DocumentSource) -> Result<Document, String> {
    validate_source(source)?;
    let mut reader = Reader {
        parser: Parser::new_ext(&source.markdown, OPTIONS),
        events: 0,
    };
    let mut blocks = reader.blocks(None, 0)?;
    let title = source.title.trim().to_owned();
    let leading_title_removed = blocks.first().is_some_and(|block| {
        matches!(block, Block::Heading { level: 1, content }
            if !title.is_empty() && super::text::inline_text(content, false).trim() == title)
    });
    if leading_title_removed {
        blocks.remove(0);
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
    Ok(Document {
        title,
        blocks,
        leading_title_removed,
    })
}

pub(super) fn validate_source(source: &DocumentSource) -> Result<(), String> {
    if source.citations.len() > 4096 {
        return Err("Document export exceeds 4096 citations".into());
    }
    let mut total = 0usize;
    for value in std::iter::once(source.title.as_str())
        .chain(std::iter::once(source.markdown.as_str()))
        .chain(
            source
                .citations
                .iter()
                .flat_map(|citation| [citation.title.as_str(), citation.quote.as_str()]),
        )
    {
        total = total.saturating_add(value.len());
        if total > MAX_SOURCE_BYTES {
            return Err("Document export input exceeds 4 MiB".into());
        }
        validate_text(value)?;
    }
    Ok(())
}

pub(super) fn validate_text(value: &str) -> Result<(), String> {
    if value.chars().any(|c| {
        (c.is_control() && !matches!(c, '\n' | '\r' | '\t')) || matches!(c, '\u{fffe}' | '\u{ffff}')
    }) {
        return Err("Document export input contains unsupported control characters".into());
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
