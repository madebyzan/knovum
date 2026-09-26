use super::document::{Block, Document, Inline};

pub(crate) fn render(document: &Document) -> String {
    let mut output = String::new();
    if !document.title.is_empty() {
        output.push_str(&document.title);
        output.push_str("\n\n");
    }
    write_blocks(&mut output, &document.blocks, "");
    output
}

pub(super) fn write_blocks(output: &mut String, blocks: &[Block], prefix: &str) {
    for block in blocks {
        match block {
            Block::Paragraph(content) | Block::Heading { content, .. } => {
                write_lines(output, &inline_text(content, true), prefix);
                output.push('\n');
            }
            Block::Code { text, .. } => {
                write_lines(output, text, prefix);
                output.push('\n');
            }
            Block::Quote(children) => write_blocks(output, children, &format!("{prefix}> ")),
            Block::List { start, items } => {
                for (index, item) in items.iter().enumerate() {
                    let marker =
                        start.map_or_else(|| "- ".into(), |n| format!("{}. ", n + index as u64));
                    let mut body = String::new();
                    write_blocks(&mut body, item, "");
                    let mut lines = body.trim_end_matches('\n').split('\n');
                    output.push_str(&format!("{prefix}{marker}{}\n", lines.next().unwrap_or("")));
                    let continuation = format!("{prefix}{}", " ".repeat(marker.len()));
                    for line in lines {
                        output.push_str(&format!("{continuation}{line}\n"));
                    }
                }
                output.push('\n');
            }
            Block::Table(table) => {
                for row in std::iter::once(&table.header).chain(&table.rows) {
                    let cells: Vec<_> = row
                        .iter()
                        .map(|cell| inline_text(cell, true).replace(['\n', '\r', '\t'], " "))
                        .collect();
                    write_lines(output, &cells.join("\t"), prefix);
                }
                output.push('\n');
            }
            Block::Rule => output.push_str(&format!("{prefix}---\n\n")),
        }
    }
}

fn write_lines(output: &mut String, text: &str, prefix: &str) {
    for line in text.split_terminator('\n') {
        output.push_str(prefix);
        output.push_str(line);
        output.push('\n');
    }
}

pub(super) fn inline_text(content: &[Inline], include_urls: bool) -> String {
    let mut output = String::new();
    for inline in content {
        match inline {
            Inline::Text(text) | Inline::Code(text) => output.push_str(text),
            Inline::Strong(children) | Inline::Emphasis(children) | Inline::Strike(children) => {
                output.push_str(&inline_text(children, include_urls));
            }
            Inline::Link { url, content } => {
                let label = inline_text(content, include_urls);
                output.push_str(&label);
                if include_urls && label != *url {
                    output.push_str(&format!(" ({url})"));
                }
            }
            Inline::Image { alt } => {
                output.push_str(&format!("[Image: {}]", inline_text(alt, false)));
            }
            Inline::Task(checked) => output.push_str(if *checked { "[x] " } else { "[ ] " }),
            Inline::SoftBreak => output.push(' '),
            Inline::HardBreak => output.push('\n'),
        }
    }
    output
}
