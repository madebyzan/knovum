mod visuals;

use super::{DocumentFileSource, DocumentFormat};
use office_oxide::ir::{
    DocumentIR, Element, InlineContent, Metadata, Paragraph, Section, TextSpan,
};
use std::collections::BTreeSet;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PdfSource {
    pub file: DocumentFileSource,
    pub recognized_pages: Vec<RecognizedPage>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecognizedPage {
    pub page: u32,
    pub text: String,
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
        let mut pages = BTreeSet::new();
        let mut text_bytes = 0;
        for page in &self.recognized_pages {
            if page.page == 0
                || page.page > 2000
                || !pages.insert(page.page)
                || page.text.trim().is_empty()
            {
                return Err("PDF recognition page identity is invalid".into());
            }
            super::super::markdown::validate_text(&page.text)?;
            text_bytes += page.text.len();
        }
        if text_bytes > 4 * 1024 * 1024 {
            return Err("PDF recognition exceeds 4 MiB".into());
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
    let page_count = document.pages.len();
    if source
        .recognized_pages
        .iter()
        .any(|page| page.page as usize > page_count)
    {
        return Err("PDF recognition refers to an unavailable page".into());
    }
    let required_pages = document
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
    for page in &source.recognized_pages {
        if !required_pages.contains(&page.page) {
            return Err("PDF recognition was supplied for a page that does not require it".into());
        }
    }
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
        let requires_recognition = required_pages.contains(&number);
        let mut elements = if requires_recognition {
            let recognized = source
                .recognized_pages
                .iter()
                .find(|value| value.page == number)
                .ok_or_else(|| {
                    format!("PDF page {number} requires recognition before conversion")
                })?;
            text_bytes += recognized.text.len();
            // OCR is literal evidence; it must never become Markdown instructions or markup.
            vec![Element::Paragraph(Paragraph {
                content: vec![InlineContent::Text(TextSpan::plain(&recognized.text))],
                ..Default::default()
            })]
        } else if page.needs_ocr {
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
            elements.extend(visuals.page(number, requires_recognition)?);
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
