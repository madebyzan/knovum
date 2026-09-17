use std::io::Cursor;

use docx_rs::{Docx, Paragraph, Run};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

pub(crate) fn render(markdown: &str) -> Result<Vec<u8>, String> {
    let mut document = Docx::new();
    let mut paragraph = Paragraph::new();
    let mut has_content = false;
    let mut heading = None;
    let mut bold = false;
    let mut italic = false;
    for event in Parser::new_ext(markdown, Options::all()) {
        match event {
            Event::Start(Tag::Heading { level, .. }) => heading = Some(level),
            Event::Start(Tag::Strong) => bold = true,
            Event::Start(Tag::Emphasis) => italic = true,
            Event::Text(text) | Event::Code(text) => {
                let mut run = Run::new().add_text(text.into_string());
                if bold {
                    run = run.bold();
                }
                if italic {
                    run = run.italic();
                }
                paragraph = paragraph.add_run(run);
                has_content = true;
            }
            Event::SoftBreak => paragraph = paragraph.add_run(Run::new().add_text(" ")),
            Event::HardBreak => {
                paragraph =
                    paragraph.add_run(Run::new().add_break(docx_rs::BreakType::TextWrapping));
            }
            Event::End(TagEnd::Strong) => bold = false,
            Event::End(TagEnd::Emphasis) => italic = false,
            Event::End(TagEnd::Heading(_))
            | Event::End(TagEnd::Paragraph)
            | Event::End(TagEnd::Item)
                if has_content =>
            {
                if let Some(level) = heading.take() {
                    paragraph = paragraph.style(heading_style(level));
                }
                document = document.add_paragraph(paragraph);
                paragraph = Paragraph::new();
                has_content = false;
            }
            _ => {}
        }
    }
    if has_content {
        document = document.add_paragraph(paragraph);
    }
    let mut cursor = Cursor::new(Vec::new());
    document
        .build()
        .pack(&mut cursor)
        .map_err(|error| error.to_string())?;
    Ok(cursor.into_inner())
}

fn heading_style(level: HeadingLevel) -> &'static str {
    match level {
        HeadingLevel::H1 => "Heading1",
        HeadingLevel::H2 => "Heading2",
        HeadingLevel::H3 => "Heading3",
        HeadingLevel::H4 => "Heading4",
        HeadingLevel::H5 => "Heading5",
        HeadingLevel::H6 => "Heading6",
    }
}
