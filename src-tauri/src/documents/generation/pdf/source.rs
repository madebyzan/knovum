use super::super::document::{Alignment, Block, CalloutKind, Document, Inline, Table};
use super::super::images::Images;

pub(super) fn render(document: &Document, images: &Images) -> Result<String, String> {
    let mut output = include_str!("document.typ").to_owned();
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
            Block::Paragraph(content) => paragraph(output, content, images)?,
            Block::Heading { level, content } => {
                output.push_str(&format!("#heading(level: {level})["));
                inlines(output, content, images)?;
                output.push(']');
            }
            Block::Quote(children) => {
                output
                    .push_str("#block(inset: (left: 12pt), stroke: (left: 1.5pt + luma(180)))[\n");
                blocks(output, children, images)?;
                output.push(']');
            }
            Block::Callout {
                kind,
                blocks: children,
            } => {
                let (bg, accent) = callout_palette(*kind);
                output.push_str(&format!(
                    "#block(width: 100%, inset: (left: 11pt, rest: 9pt), radius: 4pt, fill: {bg}, stroke: (left: 2.5pt + {accent}))[\n\
                     #text(weight: \"bold\", size: 9pt, fill: {accent})[{}]\\ \n",
                    kind.label()
                ));
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
            Block::Math(latex) => {
                let rendered = super::super::math::to_typst(latex, true);
                if !rendered.is_empty() {
                    output.push_str(&format!("#align(center)[{rendered}]"));
                }
            }
            Block::Rule => output.push_str("#line(length: 100%)"),
        }
        output.push_str("\n\n");
    }
    Ok(())
}

fn paragraph(output: &mut String, content: &[Inline], images: &Images) -> Result<(), String> {
    let mut segment: Vec<&Inline> = Vec::new();
    let flush_segment = |out: &mut String, seg: &mut Vec<&Inline>| -> Result<(), String> {
        if seg.is_empty() {
            return Ok(());
        }
        out.push_str("#par[");
        for inline in seg.drain(..) {
            write_inline(out, inline, images)?;
        }
        out.push_str("]\n\n");
        Ok(())
    };
    for inline in content {
        if let Inline::Image { destination, alt } = inline {
            flush_segment(output, &mut segment)?;
            output.push_str("#align(center)[");
            write_image(output, destination, alt, images)?;
            output.push_str("]\n\n");
        } else {
            segment.push(inline);
        }
    }
    flush_segment(output, &mut segment)
}

fn inlines(output: &mut String, content: &[Inline], images: &Images) -> Result<(), String> {
    for inline in content {
        write_inline(output, inline, images)?;
    }
    Ok(())
}

fn write_inline(output: &mut String, inline: &Inline, images: &Images) -> Result<(), String> {
    match inline {
        Inline::Text(text) => output.push_str(&format!("#text({})", string(text))),
        Inline::Code(text) => output.push_str(&format!("#raw({})", string(text))),
        Inline::Math(latex) => output.push_str(&super::super::math::to_typst(latex, false)),
        Inline::DisplayMath(latex) => output.push_str(&super::super::math::to_typst(latex, true)),
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
            output.push_str("#box[");
            write_image(output, destination, alt, images)?;
            output.push(']');
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
    Ok(())
}

fn write_image(
    output: &mut String,
    destination: &str,
    alt: &[Inline],
    images: &Images,
) -> Result<(), String> {
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

fn callout_palette(kind: CalloutKind) -> (&'static str, &'static str) {
    match kind {
        CalloutKind::Note => ("rgb(241, 246, 255)", "rgb(37, 99, 235)"),
        CalloutKind::Tip => ("rgb(239, 251, 244)", "rgb(22, 163, 74)"),
        CalloutKind::Important => ("rgb(245, 241, 255)", "rgb(124, 58, 237)"),
        CalloutKind::Warning => ("rgb(255, 249, 235)", "rgb(217, 119, 6)"),
        CalloutKind::Caution => ("rgb(254, 242, 242)", "rgb(220, 38, 38)"),
    }
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
