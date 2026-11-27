use base64::{engine::general_purpose::STANDARD, Engine};
use pulldown_cmark::{html::push_html, Event};

use super::{
    document::{Alignment, Block, DocumentSource, Inline, Table},
    images::Images,
};

/// Renders a validated Markdown document directly from the shared `Document` AST so HTML, DOCX,
/// PDF, PPTX, and TXT share one single parsing, callout, math, image, and Mermaid diagram contract.
pub(crate) fn render(source: &DocumentSource, metadata_title: &str) -> Result<Vec<u8>, String> {
    let document = super::markdown::parse(source)?;
    let images = Images::load(source)?;
    let mut output = String::new();
    if !document.title.is_empty() {
        output.push_str("<h1>");
        push_escaped_text(&mut output, &document.title);
        output.push_str("</h1>\n");
    }
    write_blocks(&mut output, &document.blocks, &images)?;
    Ok(build_document(metadata_title.trim(), &output))
}

/// Only engine-generated, validated HTML fragments enter this document shell.
pub(super) fn build_document(title: &str, body: &str) -> Vec<u8> {
    let mut escaped_title = String::new();
    push_html(&mut escaped_title, [Event::Text(title.into())].into_iter());
    format!(
        "<!doctype html>\n<html lang=\"und\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
         <meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; img-src data:; style-src 'unsafe-inline'; base-uri 'none'; form-action 'none'\">\
         <title>{escaped_title}</title><style>{}</style></head><body><main>{body}</main></body></html>\n",
        include_str!("document.css"),
    )
    .into_bytes()
}

fn write_blocks(output: &mut String, blocks: &[Block], images: &Images) -> Result<(), String> {
    for block in blocks {
        match block {
            Block::Paragraph(content) => {
                output.push_str("<p>");
                write_inlines(output, content, images)?;
                output.push_str("</p>\n");
            }
            Block::Heading { level, content } => {
                output.push_str(&format!("<h{level}>"));
                write_inlines(output, content, images)?;
                output.push_str(&format!("</h{level}>\n"));
            }
            Block::Quote(children) => {
                output.push_str("<blockquote>\n");
                write_blocks(output, children, images)?;
                output.push_str("</blockquote>\n");
            }
            Block::Callout {
                kind,
                blocks: children,
            } => {
                output.push_str(&format!(
                    "<aside class=\"callout callout-{}\"><p class=\"callout-title\">{}</p>\n",
                    kind.css_name(),
                    kind.label()
                ));
                write_blocks(output, children, images)?;
                output.push_str("</aside>\n");
            }
            Block::Code { language, text } => {
                if language.is_empty() {
                    output.push_str("<pre><code>");
                } else {
                    output.push_str("<pre><code class=\"language-");
                    push_escaped_attr(output, language);
                    output.push_str("\">");
                }
                push_escaped_text(output, text);
                output.push_str("</code></pre>\n");
            }
            Block::List { start, items } => {
                if let Some(start) = start {
                    if *start == 1 {
                        output.push_str("<ol>\n");
                    } else {
                        output.push_str(&format!("<ol start=\"{start}\">\n"));
                    }
                } else {
                    output.push_str("<ul>\n");
                }
                for item in items {
                    output.push_str("<li>");
                    write_list_item(output, item, images)?;
                    output.push_str("</li>\n");
                }
                output.push_str(if start.is_some() {
                    "</ol>\n"
                } else {
                    "</ul>\n"
                });
            }
            Block::Table(value) => write_table(output, value, images)?,
            Block::Math(latex) => {
                output.push_str(&super::math::to_mathml(latex, true));
                output.push('\n');
            }
            Block::Rule => output.push_str("<hr />\n"),
        }
    }
    Ok(())
}

fn write_list_item(output: &mut String, item: &[Block], images: &Images) -> Result<(), String> {
    if let [Block::Paragraph(content)] = item {
        write_inlines(output, content, images)?;
        return Ok(());
    }
    write_blocks(output, item, images)
}

fn write_table(output: &mut String, table: &Table, images: &Images) -> Result<(), String> {
    output.push_str("<table><thead><tr>");
    for (cell, align) in table.header.iter().zip(&table.alignments) {
        output.push_str(&format!("<th{}>", align_attr(*align)));
        write_inlines(output, cell, images)?;
        output.push_str("</th>");
    }
    output.push_str("</tr></thead><tbody>\n");
    for row in &table.rows {
        output.push_str("<tr>");
        for (cell, align) in row.iter().zip(&table.alignments) {
            output.push_str(&format!("<td{}>", align_attr(*align)));
            write_inlines(output, cell, images)?;
            output.push_str("</td>");
        }
        output.push_str("</tr>\n");
    }
    output.push_str("</tbody></table>\n");
    Ok(())
}

fn align_attr(alignment: Alignment) -> &'static str {
    match alignment {
        Alignment::Left => "",
        Alignment::Center => " style=\"text-align: center\"",
        Alignment::Right => " style=\"text-align: right\"",
    }
}

fn write_inlines(output: &mut String, content: &[Inline], images: &Images) -> Result<(), String> {
    for inline in content {
        match inline {
            Inline::Text(text) => push_escaped_text(output, text),
            Inline::Strong(children) => {
                output.push_str("<strong>");
                write_inlines(output, children, images)?;
                output.push_str("</strong>");
            }
            Inline::Emphasis(children) => {
                output.push_str("<em>");
                write_inlines(output, children, images)?;
                output.push_str("</em>");
            }
            Inline::Strike(children) => {
                output.push_str("<del>");
                write_inlines(output, children, images)?;
                output.push_str("</del>");
            }
            Inline::Code(text) => {
                output.push_str("<code>");
                push_escaped_text(output, text);
                output.push_str("</code>");
            }
            Inline::Math(latex) => output.push_str(&super::math::to_mathml(latex, false)),
            Inline::DisplayMath(latex) => output.push_str(&super::math::to_mathml(latex, true)),
            Inline::Link { url, content } => {
                super::links::validate(url)?;
                output.push_str("<a href=\"");
                push_escaped_href(output, url);
                output.push_str("\">");
                write_inlines(output, content, images)?;
                output.push_str("</a>");
            }
            Inline::Image { destination, alt } => {
                let image = images.get(destination)?;
                images.record_embedded_bytes(22 + image.bytes.len().div_ceil(3) * 4)?;
                let alt_text = super::text::inline_text(alt, false);
                output.push_str("<img src=\"data:image/png;base64,");
                output.push_str(&STANDARD.encode(&image.bytes));
                output.push_str("\" alt=\"");
                push_escaped_attr(output, &alt_text);
                output.push_str("\" />");
            }
            Inline::Task(checked) => {
                if *checked {
                    output.push_str("<input disabled=\"\" type=\"checkbox\" checked=\"\"/> ");
                } else {
                    output.push_str("<input disabled=\"\" type=\"checkbox\"/> ");
                }
            }
            Inline::SoftBreak => output.push('\n'),
            Inline::HardBreak => output.push_str("<br />\n"),
        }
    }
    Ok(())
}

fn push_escaped_text(output: &mut String, value: &str) {
    for ch in value.chars() {
        match ch {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            _ => output.push(ch),
        }
    }
}

fn push_escaped_attr(output: &mut String, value: &str) {
    for ch in value.chars() {
        match ch {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            _ => output.push(ch),
        }
    }
}

fn push_escaped_href(output: &mut String, value: &str) {
    for ch in value.chars() {
        match ch {
            '&' => output.push_str("&amp;"),
            '"' => output.push_str("%22"),
            '<' => output.push_str("%3C"),
            '>' => output.push_str("%3E"),
            ' ' => output.push_str("%20"),
            _ => output.push(ch),
        }
    }
}
