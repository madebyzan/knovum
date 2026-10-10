mod validation;

use std::io::Cursor;

use office_oxide::{core::xml, Document, DocumentFormat as OfficeFormat};

use super::{DocumentFileSource, DocumentFormat};

pub(super) fn render(
    source: &DocumentFileSource,
    bytes: Vec<u8>,
    format: DocumentFormat,
) -> Result<Vec<u8>, String> {
    let input_format =
        OfficeFormat::from_path(&source.path).ok_or("Unsupported Office source format")?;
    crate::documents::extraction::office::budget::preflight_archive(&bytes, input_format)?;
    xml::reset_truncated_subtrees();
    let document = Document::from_reader(Cursor::new(bytes), input_format)
        .map_err(|error| format!("Office conversion parsing failed: {error}"))?;
    let ir = document.to_ir();
    validation::validate(&ir, format)?;
    let output = match format {
        DocumentFormat::Docx => {
            validate_writer_completeness()?;
            let writer = office_oxide::create::ir_to_docx(&ir);
            let mut output = Cursor::new(Vec::new());
            writer
                .write_to(&mut output)
                .map_err(|error| error.to_string())?;
            output.into_inner()
        }
        DocumentFormat::Pptx => {
            if input_format != OfficeFormat::Ppt {
                return Err("Office slide conversion requires a legacy PowerPoint source".into());
            }
            if ir.sections.len() > 100 {
                return Err("Office presentation conversion exceeds 100 slides".into());
            }
            let writer = office_oxide::create::ir_to_pptx(&ir);
            let mut output = Cursor::new(Vec::new());
            writer
                .write_to(&mut output)
                .map_err(|error| error.to_string())?;
            output.into_inner()
        }
        DocumentFormat::Html => super::super::html::build_document(
            &source.title,
            &ir.to_html_with(office_oxide::ir_render::HtmlOptions {
                image_embed: office_oxide::ir_render::ImageEmbed::Base64,
            }),
        ),
        DocumentFormat::Text => ir.plain_text().into_bytes(),
        DocumentFormat::Pdf => format.render(&super::super::document::DocumentSource {
            title: String::new(),
            markdown: ir.to_markdown(),
            citations: vec![],
        })?,
    };
    validate_writer_completeness()?;
    Ok(output)
}

fn validate_writer_completeness() -> Result<(), String> {
    if xml::truncated_subtrees() != 0 {
        return Err("Office conversion exceeded the writer nesting budget".into());
    }
    Ok(())
}
