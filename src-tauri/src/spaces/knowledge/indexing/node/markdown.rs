use serde_json::Value;

use crate::spaces::knowledge::indexing::{self, KnowledgeChunkSeed};

pub(super) fn markdown_chunks(text: &str, revision: &str) -> Vec<KnowledgeChunkSeed> {
    if text.trim().is_empty() {
        return Vec::new();
    }
    let lines = line_records(text);
    let mut sections = Vec::new();
    let mut section_start = 0usize;
    let mut heading: Option<String> = None;
    for index in 0..=lines.len() {
        let next_heading = lines.get(index).and_then(|line| parse_heading(line.text));
        if index == lines.len() || next_heading.is_some() {
            if index > section_start {
                let start_byte = lines[section_start].start_byte;
                let end_byte = lines
                    .get(index)
                    .map(|line| line.start_byte)
                    .unwrap_or(text.len());
                push_section(
                    &mut sections,
                    text,
                    start_byte,
                    end_byte,
                    heading.as_deref(),
                    revision,
                );
            }
            if index == lines.len() {
                break;
            }
            section_start = index;
            heading = next_heading
                .map(|value| crate::documents::extraction::text::bounded_utf16_text(&value, 512));
        }
    }
    if sections.is_empty() {
        push_section(&mut sections, text, 0, text.len(), None, revision);
    }
    sections
}

fn push_section(
    output: &mut Vec<KnowledgeChunkSeed>,
    source: &str,
    start_byte: usize,
    end_byte: usize,
    heading: Option<&str>,
    revision: &str,
) {
    let raw = &source[start_byte..end_byte];
    let leading = raw.len() - raw.trim_start().len();
    let text = raw.trim();
    if text.is_empty() {
        return;
    }
    let exact_byte = start_byte + leading;
    let base_utf16 = source[..exact_byte].encode_utf16().count();
    for (part, passage) in indexing::split_passages_with_offsets(text)
        .into_iter()
        .enumerate()
    {
        let mut location = serde_json::json!({
            "kind":"markdown",
            "startOffset": base_utf16 + passage.start_offset,
            "endOffset": base_utf16 + passage.end_offset,
        });
        let mut metadata = serde_json::json!({"part": part});
        if let Some(heading) = heading {
            location["heading"] = Value::String(heading.to_owned());
            metadata["heading"] = Value::String(heading.to_owned());
        }
        output.push(KnowledgeChunkSeed {
            kind: if heading.is_some() { "heading" } else { "text" },
            locator: indexing::locator(revision, location, &passage.text),
            metadata,
            text: passage.text,
        });
    }
}

struct LineRecord<'a> {
    text: &'a str,
    start_byte: usize,
}

fn line_records(text: &str) -> Vec<LineRecord<'_>> {
    let mut records = Vec::new();
    let mut cursor = 0usize;
    while cursor < text.len() {
        let newline = text[cursor..]
            .find('\n')
            .map(|index| cursor + index)
            .unwrap_or(text.len());
        let end = if newline > cursor && text.as_bytes()[newline - 1] == b'\r' {
            newline - 1
        } else {
            newline
        };
        records.push(LineRecord {
            text: &text[cursor..end],
            start_byte: cursor,
        });
        if newline == text.len() {
            break;
        }
        cursor = newline + 1;
    }
    records
}

fn parse_heading(line: &str) -> Option<String> {
    use pulldown_cmark::{Event, Parser, Tag, TagEnd};
    let trimmed = line.trim();
    if !trimmed.starts_with('#') {
        return None;
    }
    let mut parser = Parser::new(trimmed);
    match parser.next()? {
        Event::Start(Tag::Heading { .. }) => {
            let mut text = String::new();
            for event in parser {
                match event {
                    Event::Text(t) | Event::Code(t) => text.push_str(&t),
                    Event::End(TagEnd::Heading(_)) => break,
                    _ => {}
                }
            }
            let content = text.trim().to_owned();
            (!content.is_empty()).then_some(content)
        }
        _ => None,
    }
}
