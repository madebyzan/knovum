pub(crate) mod validation;

use std::io::Cursor;

use office_oxide::{core::xml, Document, DocumentFormat as OfficeFormat};

use super::{DocumentFileSource, DocumentFormat};

/// Reads an Office resource into the authoring model, which is then serialized the same way every
/// other document is: one reading, one writer per format. Slides are the exception, because they
/// are only slides when the source was a presentation.
pub(super) fn render(
    source: &DocumentFileSource,
    bytes: Vec<u8>,
    format: DocumentFormat,
) -> Result<Vec<u8>, String> {
    let input_format =
        OfficeFormat::from_path(&source.path).ok_or("Unsupported Office source format")?;
    if format == DocumentFormat::Pptx && input_format != OfficeFormat::Ppt {
        return Err("Office slide conversion requires a legacy PowerPoint source".into());
    }
    crate::documents::extraction::office::budget::preflight_archive(&bytes, input_format)?;
    xml::reset_truncated_subtrees();
    let document = Document::from_reader(Cursor::new(bytes), input_format)
        .map_err(|error| format!("Office conversion parsing failed: {error}"))?;
    let ir = document.to_ir();
    if xml::truncated_subtrees() != 0 {
        return Err("Office conversion exceeded the reader nesting budget".into());
    }
    if format == DocumentFormat::Pptx && ir.sections.len() > 100 {
        return Err("Office presentation conversion exceeds 100 slides".into());
    }
    super::super::structured::render(ir, format, &source.title)
}
