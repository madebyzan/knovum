use std::io::Cursor;

use office_oxide::{ir::Metadata, pptx::write::PptxWriter, DocumentFormat};

use super::{
    document::{Block, DocumentSource},
    markdown, text,
};

pub(crate) const ENGINE: &str = "office-oxide-pptx:0.1.13";

pub(crate) fn render(source: &DocumentSource) -> Result<Vec<u8>, String> {
    let slides = slide_sections(source)?;
    let mut writer = PptxWriter::new();
    writer.set_metadata(&Metadata {
        format: DocumentFormat::Pptx,
        title: Some(source.title.clone()),
        author: Some("Knovum".into()),
        ..Metadata::default()
    });

    for (heading, body) in slides {
        let slide = writer.add_slide();
        slide.set_title(&heading);
        if !body.is_empty() {
            slide.add_text(&body);
        }
    }

    let mut output = Cursor::new(Vec::new());
    writer
        .write_to(&mut output)
        .map_err(|error| format!("presentation-write: {error}"))?;
    Ok(output.into_inner())
}

fn slide_sections(source: &DocumentSource) -> Result<Vec<(String, String)>, String> {
    let document = markdown::parse(source)?;
    let mut slides = vec![(document.title, String::new())];
    let mut heading = "Overview".to_owned();
    let mut body = String::new();
    for block in &document.blocks {
        if let Block::Heading { content, .. } = block {
            if !body.is_empty() || heading != "Overview" {
                slides.push((heading, std::mem::take(&mut body)));
            }
            heading = text::inline_text(content, false);
        } else {
            text::write_blocks(&mut body, std::slice::from_ref(block), "");
        }
    }
    if !body.is_empty() || heading != "Overview" {
        slides.push((heading, body));
    }
    if slides.len() > 100 {
        return Err("presentation-slide-limit: presentation exceeds 100 slides".into());
    }
    Ok(slides)
}
