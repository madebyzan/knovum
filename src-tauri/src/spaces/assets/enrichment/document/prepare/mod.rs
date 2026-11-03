use std::collections::BTreeSet;
use std::path::Path;

use tauri::AppHandle;

use super::resource;
use crate::documents::{
    extraction::{pdf::PdfDocumentContent, pdf_visuals::PdfVisual, visual::DocumentImage},
    pdf_input::PdfInput,
};
use crate::spaces::knowledge::model::document_coverage::{
    DocumentCoverage, DocumentOccurrence, DocumentParser, MAX_COVERED_PAGES, MAX_COVERED_VISUALS,
    MAX_TOTAL_RESOURCE_BYTES,
};

/// What one coverage record describes of a document, and what it had to leave out.
struct Covered<'a> {
    recognition: Vec<u32>,
    extracted: Vec<u32>,
    empty: Vec<u32>,
    visuals: Vec<&'a PdfVisual>,
    warnings: Vec<String>,
}

fn keep_up_to(list: &[u32], limit: Option<u32>) -> Vec<u32> {
    list.iter()
        .copied()
        .filter(|page| limit.is_none_or(|boundary| *page < boundary))
        .collect()
}

/// Whole-page recognition inputs and page classifications take priority over sub-page image crops.
///
/// A page claiming recognition and the `PdfInput::Page` that recognises it are the same fact stated
/// twice, so they share `recognition_limit`. Sub-page `PdfInput::Image` crops consume whatever
/// visual budget remains up to a whole-page `image_limit` without truncating page coverage or OCR
/// of later scanned pages.
fn covered(document: &PdfDocumentContent) -> Covered<'_> {
    let recognition_limit = [
        document
            .visuals
            .iter()
            .filter(|visual| matches!(visual.input, PdfInput::Page { .. }))
            .nth(MAX_COVERED_VISUALS)
            .map(PdfVisual::page),
        document
            .pages_requiring_recognition
            .get(MAX_COVERED_PAGES)
            .copied(),
    ]
    .into_iter()
    .flatten()
    .min();
    let recognition = keep_up_to(&document.pages_requiring_recognition, recognition_limit);

    let remaining_pages = MAX_COVERED_PAGES.saturating_sub(recognition.len());
    let non_recognition = document
        .pages_with_extracted_text
        .iter()
        .chain(&document.pages_without_extractable_content)
        .copied()
        .collect::<BTreeSet<_>>();
    let page_limit = non_recognition.iter().nth(remaining_pages).copied();
    let extracted = keep_up_to(&document.pages_with_extracted_text, page_limit);
    let empty = keep_up_to(&document.pages_without_extractable_content, page_limit);

    let recognition_visuals = document
        .visuals
        .iter()
        .filter(|visual| {
            matches!(visual.input, PdfInput::Page { .. })
                && recognition_limit.is_none_or(|boundary| visual.page() < boundary)
        })
        .count();
    let remaining_visuals = MAX_COVERED_VISUALS.saturating_sub(recognition_visuals);
    let image_limit = document
        .visuals
        .iter()
        .filter(|visual| matches!(visual.input, PdfInput::Image { .. }))
        .nth(remaining_visuals)
        .map(PdfVisual::page);

    let visuals = document
        .visuals
        .iter()
        .filter(|visual| match visual.input {
            PdfInput::Page { .. } => {
                recognition_limit.is_none_or(|boundary| visual.page() < boundary)
            }
            PdfInput::Image { .. } => image_limit.is_none_or(|boundary| visual.page() < boundary),
        })
        .collect::<Vec<_>>();

    let mut warnings = vec!["pdf_visual_coverage_limited".to_owned()];
    if visuals.len() < document.visuals.len() {
        warnings.push("pdf_visual_occurrence_coverage_limited".to_owned());
    }
    let claimed_pages = document.pages_requiring_recognition.len() + non_recognition.len();
    if recognition.len() + extracted.len() + empty.len() < claimed_pages {
        warnings.push("pdf_page_coverage_limited".to_owned());
    }
    if document.text_truncated {
        warnings.push("pdf_text_coverage_limited".to_owned());
    }
    Covered {
        recognition,
        extracted,
        empty,
        visuals,
        warnings,
    }
}

pub(in crate::spaces::assets::enrichment) fn pdf(
    app: &AppHandle,
    source: &Path,
    document: &PdfDocumentContent,
    revision: &str,
    cancellation: &tokio_util::sync::CancellationToken,
    claim_managed_path: &mut dyn FnMut(&Path) -> Result<(), String>,
) -> Result<DocumentCoverage, String> {
    let covered = covered(document);
    let mut report = DocumentCoverage {
        schema_version: 1,
        parser: DocumentParser::PdfInspector,
        source_revision: revision.into(),
        page_count: Some(document.page_count),
        pages_requiring_recognition: covered.recognition,
        pages_with_extracted_text: covered.extracted,
        pages_without_extractable_content: covered.empty,
        resources: Vec::new(),
        occurrences: Vec::new(),
        warnings: covered.warnings,
    };
    for (index, visual) in covered.visuals.iter().enumerate() {
        report.occurrences.push(DocumentOccurrence {
            id: format!("pdf:{}:{index}", visual.input.page()),
            location: visual.input.location(),
            context: visual.context.clone(),
            alt_text: None,
            resource_id: None,
            pdf_input: Some(visual.input.clone()),
            unavailable: None,
        });
    }
    super::raster::materialize_all(app, source, &mut report, cancellation, claim_managed_path)?;
    report.validate()?;
    Ok(report)
}

pub(in crate::spaces::assets::enrichment) fn office(
    app: &AppHandle,
    source: &Path,
    images: Vec<DocumentImage>,
    warnings: Vec<String>,
    revision: &str,
    cancellation: &tokio_util::sync::CancellationToken,
    claim_managed_path: &mut dyn FnMut(&Path) -> Result<(), String>,
) -> Result<DocumentCoverage, String> {
    let mut report = DocumentCoverage {
        schema_version: 1,
        parser: DocumentParser::OfficeOxide,
        source_revision: revision.into(),
        page_count: None,
        pages_requiring_recognition: Vec::new(),
        pages_with_extracted_text: Vec::new(),
        pages_without_extractable_content: Vec::new(),
        resources: Vec::new(),
        occurrences: Vec::new(),
        warnings,
    };
    let mut resource_ids = std::collections::BTreeSet::new();
    let mut resource_bytes = 0_u64;
    for image in images {
        crate::spaces::assets::enrichment::ensure_active(cancellation)?;
        let mut occurrence = DocumentOccurrence {
            id: image.id,
            location: image.location,
            context: image.context,
            alt_text: image.alt_text,
            resource_id: None,
            pdf_input: None,
            unavailable: None,
        };
        match image
            .data
            .as_deref()
            .map(|bytes| resource::save(app, source, bytes, claim_managed_path))
        {
            Some(Ok(resource)) => {
                occurrence.resource_id = Some(resource.id.clone());
                if resource_ids.insert(resource.id.clone()) {
                    resource_bytes += resource.byte_size;
                    if resource_bytes > MAX_TOTAL_RESOURCE_BYTES {
                        return Err("Document visual resources exceed 256 MiB".into());
                    }
                    report.resources.push(resource);
                }
            }
            Some(Err(resource::ResourceError::Unsupported(error))) => {
                occurrence.unavailable = Some(error);
            }
            Some(Err(resource::ResourceError::Storage(error))) => {
                return Err(error);
            }
            None => {
                occurrence.unavailable =
                    Some("Referenced image bytes are unavailable from parser".into());
            }
        }
        report.occurrences.push(occurrence);
    }
    report.validate()?;
    Ok(report)
}

#[cfg(test)]
mod tests;
