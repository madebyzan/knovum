use std::io::Cursor;

use office_oxide::{
    core::xml,
    ir::{DocumentIR, Element, Metadata, Section},
    DocumentFormat as OfficeFormat,
};

use super::DocumentFormat;

/// One section of elements, as a document every writer accepts. Whoever reads a resource into
/// elements says what it holds; how a document is framed is said once, here.
pub(super) fn document(title: Option<String>, elements: Vec<Element>) -> DocumentIR {
    DocumentIR {
        metadata: Metadata {
            format: OfficeFormat::Docx,
            title,
            author: Some("Knovum".into()),
            ..Default::default()
        },
        sections: vec![Section {
            elements,
            ..Default::default()
        }],
        defined_names: Vec::new(),
    }
}

/// Rich Notes already have an authoring model. Preserve its vendor IR through serialization.
pub(super) fn render(
    mut ir: DocumentIR,
    format: DocumentFormat,
    title: &str,
) -> Result<Vec<u8>, String> {
    super::input::office::validation::validate(&ir, format)?;
    xml::reset_truncated_subtrees();
    let output = match format {
        DocumentFormat::Docx => {
            super::docx::image_layout::prepare(&mut ir)?;
            let writer = office_oxide::create::ir_to_docx(&ir);
            let mut bytes = Cursor::new(Vec::new());
            writer
                .write_to(&mut bytes)
                .map_err(|error| error.to_string())?;
            bytes.into_inner()
        }
        DocumentFormat::Pdf => super::pdf::render_ir(&ir)?,
        DocumentFormat::Html => super::html::build_document(
            title,
            &ir.to_html_with(office_oxide::ir_render::HtmlOptions {
                image_embed: office_oxide::ir_render::ImageEmbed::Base64,
            }),
        ),
        DocumentFormat::Text => ir.plain_text().into_bytes(),
        DocumentFormat::Pptx => super::pptx::render_ir(&ir)?,
    };
    if xml::truncated_subtrees() != 0 {
        return Err("Document writing exceeded the nesting budget".into());
    }
    Ok(output)
}
