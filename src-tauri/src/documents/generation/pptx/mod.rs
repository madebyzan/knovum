mod pagination;
mod paragraphs;
mod tables;
mod titles;
mod writing;

use super::document::DocumentSource;
use office_oxide::ir::DocumentIR;
use std::io::Cursor;

pub(crate) const ENGINE: &str = "office-oxide-pptx:0.1.13;knovum-presentation:2";

pub(crate) fn render(source: &DocumentSource) -> Result<Vec<u8>, String> {
    let document = super::markdown::parse(source)?;
    let images = super::images::Images::load(source)?;
    render_ir(&super::markdown::ir::translate(&document, &images)?)
}

pub(super) fn render_ir(ir: &DocumentIR) -> Result<Vec<u8>, String> {
    let slides = pagination::paginate(ir)?;
    office_oxide::core::xml::reset_truncated_subtrees();
    let mut writer = office_oxide::pptx::write::PptxWriter::new();
    writer.set_metadata(&slides.metadata);
    writer.set_presentation_size(12_192_000, 6_858_000);
    for section in &slides.sections {
        writing::slide(writer.add_slide(), section)?;
    }
    let mut output = Cursor::new(Vec::new());
    writer
        .write_to(&mut output)
        .map_err(|error| format!("presentation-write: {error}"))?;
    if office_oxide::core::xml::truncated_subtrees() != 0 {
        return Err("Presentation writing exceeded the nesting budget".into());
    }
    Ok(output.into_inner())
}

#[cfg(test)]
mod tests;
