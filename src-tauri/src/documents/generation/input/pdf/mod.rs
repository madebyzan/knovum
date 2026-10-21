mod visuals;

use super::{DocumentFileSource, DocumentFormat};
use office_oxide::ir::{DocumentIR, Metadata, Section};
use std::collections::BTreeSet;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PdfSource {
    pub file: DocumentFileSource,
}

impl PdfSource {
    pub(super) fn validate(&self) -> Result<(), String> {
        self.file.validate()?;
        if !self
            .file
            .path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("pdf"))
        {
            return Err("PDF input requires a PDF resource".into());
        }
        Ok(())
    }
}

pub(super) fn render(source: &PdfSource, format: DocumentFormat) -> Result<Vec<u8>, String> {
    source.validate()?;
    if matches!(format, DocumentFormat::Pdf) {
        return Err("Document already has the requested format".into());
    }
    let bytes = super::file::read(&source.file)?;
    let document = pdf_inspector::extract_pages_markdown_mem(&bytes, None)
        .map_err(|error| format!("PDF conversion parsing failed: {error}"))?;
    if document.pages.is_empty() || document.pages.len() > 2000 {
        return Err("PDF conversion requires between 1 and 2000 pages".into());
    }
    // A scanned page carries no text layer, so it is kept as its page image.
    let scanned_pages = document
        .pages
        .iter()
        .filter_map(|page| {
            match crate::documents::pdf_input::PdfRecognitionReason::from_parser(
                page.ocr_reason.as_deref(),
            ) {
                Ok(Some(_)) => Some(Ok(page.page + 1)),
                Ok(None) => None,
                Err(error) => Some(Err(error)),
            }
        })
        .collect::<Result<BTreeSet<_>, String>>()?;
    let mut visuals = if matches!(format, DocumentFormat::Text) {
        None
    } else {
        Some(visuals::Visuals::new(&bytes)?)
    };
    let mut ir = DocumentIR {
        metadata: Metadata {
            title: if matches!(format, DocumentFormat::Pptx) {
                Some(source.file.title.clone())
            } else {
                None
            },
            ..Default::default()
        },
        ..Default::default()
    };
    let mut text_bytes = 0;
    for page in document.pages {
        let number = page.page + 1;
        let scanned = scanned_pages.contains(&number);
        let mut elements = if scanned || page.needs_ocr {
            vec![]
        } else {
            text_bytes += page.markdown.len();
            let parsed = super::super::markdown::parse(&super::super::document::DocumentSource {
                markdown: page.markdown,
                ..Default::default()
            })?;
            super::super::markdown::ir::translate(
                &parsed,
                &super::super::images::Images::from_sources(&[])?,
            )?
            .sections
            .into_iter()
            .flat_map(|section| section.elements)
            .collect()
        };
        if text_bytes > 4 * 1024 * 1024 {
            return Err("PDF conversion text exceeds 4 MiB".into());
        }
        if let Some(visuals) = &mut visuals {
            elements.extend(visuals.page(number, scanned)?);
        }
        ir.sections.push(Section {
            elements,
            ..Default::default()
        });
    }
    if ir
        .sections
        .iter()
        .all(|section| section.elements.is_empty())
    {
        return Err("PDF has no convertible content".into());
    }
    super::super::structured::render(ir, format, &source.file.title)
}
