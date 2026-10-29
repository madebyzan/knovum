//! Splits a stored answer into the sentences verification judges.
//!
//! Only prose is checkable: paragraphs, list items, table cells and block quotes. Headings, code
//! raw HTML and table headers are not statements. Every statement keeps its byte range in the answer Markdown,
//! which is what the reader marks, and the citation markers that end it: a marker belongs to the
//! sentence it follows, so `… grew 12%. [C2]` cites the sentence about growth.

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use unicode_segmentation::UnicodeSegmentation;

use super::contracts::AnswerStatement;

/// Words verification reads; the rest of a longer answer is reported as not checked.
pub(crate) const MAX_WORDS: usize = 6_000;
/// Statements one verification judges.
pub(crate) const MAX_STATEMENTS: usize = 160;

pub(crate) struct Segmentation {
    pub statements: Vec<AnswerStatement>,
    pub truncated: bool,
}

pub(crate) fn segment(markdown: &str) -> Segmentation {
    let mut blocks = Blocks::default();
    let mut skipped_depth = 0usize;
    for (event, range) in Parser::new_ext(markdown, Options::ENABLE_TABLES).into_offset_iter() {
        match event {
            Event::Start(tag) if skips(&tag) => {
                blocks.flush();
                skipped_depth += 1;
            }
            Event::End(tag) if skipped_end(tag) => {
                skipped_depth = skipped_depth.saturating_sub(1);
            }
            Event::Start(tag) if bounds(&tag) => blocks.flush(),
            Event::End(tag) if bounded_end(tag) => blocks.flush(),
            _ if skipped_depth > 0 => {}
            Event::Text(_) | Event::Code(_) => blocks.push(&markdown[range.clone()], range.start),
            Event::SoftBreak | Event::HardBreak => blocks.push_space(range.start),
            _ => {}
        }
    }
    blocks.flush();
    blocks.finish()
}

fn skips(tag: &Tag) -> bool {
    matches!(
        tag,
        Tag::Heading { .. }
            | Tag::CodeBlock(_)
            | Tag::HtmlBlock
            | Tag::Image { .. }
            | Tag::TableHead
    )
}

fn skipped_end(tag: TagEnd) -> bool {
    matches!(
        tag,
        TagEnd::Heading(_)
            | TagEnd::CodeBlock
            | TagEnd::HtmlBlock
            | TagEnd::Image
            | TagEnd::TableHead
    )
}

fn bounds(tag: &Tag) -> bool {
    matches!(
        tag,
        Tag::Paragraph | Tag::Item | Tag::TableCell | Tag::BlockQuote(_) | Tag::List(_)
    )
}

fn bounded_end(tag: TagEnd) -> bool {
    matches!(
        tag,
        TagEnd::Paragraph
            | TagEnd::Item
            | TagEnd::TableCell
            | TagEnd::BlockQuote(_)
            | TagEnd::List(_)
    )
}

/// The prose of one block as plain text, with the source offset behind every plain byte.
#[derive(Default)]
struct Blocks {
    text: String,
    origin: Vec<usize>,
    statements: Vec<AnswerStatement>,
    words: usize,
    truncated: bool,
}

impl Blocks {
    fn push(&mut self, source: &str, offset: usize) {
        for (index, character) in source.char_indices() {
            self.text.push(character);
            self.origin
                .extend((0..character.len_utf8()).map(|byte| offset + index + byte));
        }
    }

    fn push_space(&mut self, offset: usize) {
        self.text.push(' ');
        self.origin.push(offset);
    }

    fn flush(&mut self) {
        let text = std::mem::take(&mut self.text);
        let origin = std::mem::take(&mut self.origin);
        if self.truncated || text.trim().is_empty() {
            return;
        }
        let (prose, markers) = strip_markers(&text);
        for (offset, sentence) in prose.text.split_sentence_bound_indices() {
            let body = sentence.trim();
            if !is_checkable(body) {
                continue;
            }
            self.words += body.unicode_words().count();
            if self.words > MAX_WORDS || self.statements.len() >= MAX_STATEMENTS {
                self.truncated = true;
                return;
            }
            let start = offset + sentence.len() - sentence.trim_start().len();
            let end = start + body.len();
            // A marker belongs to the sentence it follows, up to the next sentence's first letter.
            let mut citations = Vec::new();
            extend_unique(
                &mut citations,
                markers
                    .iter()
                    .filter(|marker| marker.at > start && marker.at <= offset + sentence.len())
                    .map(|marker| marker.index),
            );
            self.statements.push(AnswerStatement {
                id: self.statements.len() as u32 + 1,
                text: body.to_owned(),
                start: origin[prose.origin[start]],
                end: origin[prose.origin[end - 1]] + 1,
                citations,
            });
        }
    }

    fn finish(self) -> Segmentation {
        Segmentation {
            statements: self.statements,
            truncated: self.truncated,
        }
    }
}

/// A sentence worth judging carries at least a few letters, not just a list number or a symbol.
fn is_checkable(sentence: &str) -> bool {
    sentence.chars().filter(|c| c.is_alphanumeric()).count() >= 4
}

fn extend_unique(target: &mut Vec<u32>, values: impl Iterator<Item = u32>) {
    for value in values {
        if !target.contains(&value) {
            target.push(value);
        }
    }
}

struct Marker {
    index: u32,
    /// Byte offset in the marker-free text where the marker stood.
    at: usize,
}

/// Plain text without its citation markers, with the offset of each kept byte in the original.
struct Prose {
    text: String,
    origin: Vec<usize>,
}

impl Prose {
    fn trim_end(&mut self) {
        let kept = self.text.trim_end().len();
        self.text.truncate(kept);
        self.origin.truncate(kept);
    }
}

fn is_closing(character: char) -> bool {
    matches!(
        character,
        '.' | ',' | ';' | ':' | '!' | '?' | ')' | '。' | '，' | '；' | '：' | '！' | '？' | '）'
    )
}

/// Removes `[Cn]` markers, remembering where each stood, so they neither split sentences nor
/// count as words. A marker takes the whitespace before it, so `flat [C1].` reads `flat.`.
fn strip_markers(text: &str) -> (Prose, Vec<Marker>) {
    let mut prose = Prose {
        text: String::with_capacity(text.len()),
        origin: Vec::with_capacity(text.len()),
    };
    let mut markers = Vec::new();
    let mut cursor = 0;
    while cursor < text.len() {
        if let Some((index, length)) = marker_at(&text[cursor..]) {
            markers.push(Marker {
                index,
                at: prose.text.trim_end().len(),
            });
            cursor += length;
            let next = text[cursor..].chars().next();
            if next.is_none_or(|c| c.is_whitespace() || is_closing(c)) {
                prose.trim_end();
            }
            continue;
        }
        let character = text[cursor..].chars().next().unwrap_or(' ');
        prose.text.push(character);
        prose
            .origin
            .extend((0..character.len_utf8()).map(|byte| cursor + byte));
        cursor += character.len_utf8();
    }
    (prose, markers)
}

/// The citation ordinal and byte length of a `[Cn]` marker starting `text`.
fn marker_at(text: &str) -> Option<(u32, usize)> {
    let digits = text.strip_prefix("[C")?;
    let length = digits.bytes().take_while(u8::is_ascii_digit).count();
    if length == 0 || !digits[length..].starts_with(']') {
        return None;
    }
    let index = digits[..length].parse().ok().filter(|index| *index > 0)?;
    Some((index, length + 3))
}

#[cfg(test)]
#[path = "tests/segmentation.rs"]
mod tests;
