use super::super::document::{Alignment, Block, Document, Inline, Table};
use super::super::images::Images;
use crate::desktop::system_font::SYSTEM_SANS_FAMILY;

pub(super) fn render(document: &Document, images: &Images) -> Result<String, String> {
    let mut output = include_str!("document.typ").to_owned();
    output.push_str(&format!(
        "#set text(font: {}, size: 10.5pt)\n",
        string(SYSTEM_SANS_FAMILY)
    ));
    if !document.title.is_empty() {
        output.push_str(&format!(
            "#heading(level: 1, outlined: false)[#text({})]\n",
            string(&document.title)
        ));
    }
    blocks(&mut output, &document.blocks, images)?;
    Ok(output)
}

fn blocks(output: &mut String, content: &[Block], images: &Images) -> Result<(), String> {
    for block in content {
        match block {
            Block::Paragraph(content) => {
                output.push_str("#par[");
                inlines(output, content, images)?;
                output.push(']');
            }
            Block::Heading { level, content } => {
                output.push_str(&format!("#heading(level: {level})["));
                inlines(output, content, images)?;
                output.push(']');
            }
            Block::Quote(children) => {
                output.push_str("#block(inset: (left: 12pt), stroke: (left: 1pt + luma(180)))[\n");
                blocks(output, children, images)?;
                output.push(']');
            }
            Block::Code { language, text } => {
                output.push_str(&format!(
                    "#raw({}, block: true, lang: {})",
                    string(text),
                    string(language)
                ));
            }
            Block::List { start, items } => {
                if let Some(start) = start {
                    output.push_str(&format!("#enum(start: {start}, tight: false,\n"));
                } else {
                    output.push_str("#list(tight: false,\n");
                }
                for item in items {
                    output.push_str("[\n");
                    blocks(output, item, images)?;
                    output.push_str("],\n");
                }
                output.push(')');
            }
            Block::Table(value) => table(output, value, images)?,
            Block::Rule => output.push_str("#line(length: 100%)"),
        }
        output.push_str("\n\n");
    }
    Ok(())
}

fn inlines(output: &mut String, content: &[Inline], images: &Images) -> Result<(), String> {
    for inline in content {
        match inline {
            Inline::Text(text) => output.push_str(&format!("#text({})", string(text))),
            Inline::Code(text) => output.push_str(&format!("#raw({})", string(text))),
            Inline::Strong(children) => span(output, "strong", children, images)?,
            Inline::Emphasis(children) => span(output, "emph", children, images)?,
            Inline::Strike(children) => span(output, "strike", children, images)?,
            Inline::Link { url, content } => {
                super::super::links::validate(url)?;
                output.push_str(&format!("#link({})[", string(url)));
                inlines(output, content, images)?;
                output.push(']');
            }
            Inline::Image { destination, alt } => {
                let name = images.filename(destination)?;
                let label = super::super::text::inline_text(alt, false);
                let image = images.get(destination)?;
                let scale = (468.0 / f64::from(image.width))
                    .min(595.0 / f64::from(image.height))
                    .min(0.75);
                output.push_str(&format!(
                    "#image({}, width: {}pt, alt: {})",
                    string(&name),
                    f64::from(image.width) * scale,
                    string(&label)
                ));
            }
            Inline::Task(checked) => {
                output.push_str(if *checked {
                    "#text(\"[x] \")"
                } else {
                    "#text(\"[ ] \")"
                });
            }
            Inline::SoftBreak => output.push_str("#text(\" \")"),
            Inline::HardBreak => output.push_str("#linebreak()"),
        }
    }
    Ok(())
}

fn span(
    output: &mut String,
    name: &str,
    content: &[Inline],
    images: &Images,
) -> Result<(), String> {
    output.push_str(&format!("#{name}["));
    inlines(output, content, images)?;
    output.push(']');
    Ok(())
}

fn table(output: &mut String, table: &Table, images: &Images) -> Result<(), String> {
    output.push_str(&format!(
        "#table(columns: {}, align: (",
        table.alignments.len()
    ));
    for alignment in &table.alignments {
        output.push_str(match alignment {
            Alignment::Left => "left,",
            Alignment::Center => "center,",
            Alignment::Right => "right,",
        });
    }
    output.push_str("), table.header(");
    for cell in &table.header {
        output.push_str("[#strong[");
        inlines(output, cell, images)?;
        output.push_str("]],");
    }
    output.push_str("),\n");
    for row in &table.rows {
        for cell in row {
            output.push('[');
            inlines(output, cell, images)?;
            output.push_str("],");
        }
        output.push('\n');
    }
    output.push(')');
    Ok(())
}

// User text is always a quoted argument, never interpolated Typst markup or code.
pub(super) fn string(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t");
    format!("\"{escaped}\"")
}
