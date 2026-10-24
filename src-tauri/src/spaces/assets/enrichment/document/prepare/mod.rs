use std::collections::BTreeSet;
use std::path::Path;

use tauri::AppHandle;

use super::resource;
use crate::documents::extraction::{
    pdf::PdfDocumentContent, pdf_visuals::PdfVisual, visual::DocumentImage,
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

/// The record describes a document completely up to one page and not at all beyond it.
///
/// A page claiming recognition and the input that recognises it are the same fact stated twice, so
/// a record that kept one and dropped the other would not validate — and a reader could not tell
/// which half to believe. One boundary, derived from whichever budget runs out first, keeps the
/// record true about the part of the document it describes.
fn covered(document: &PdfDocumentContent) -> Covered<'_> {
    let pages = [
        &document.pages_requiring_recognition,
        &document.pages_with_extracted_text,
        &document.pages_without_extractable_content,
    ];
    let claimed = pages
        .iter()
        .flat_map(|list| list.iter().copied())
        .collect::<BTreeSet<_>>();
    let limit = [
        document
            .visuals
            .get(MAX_COVERED_VISUALS)
            .map(PdfVisual::page),
        claimed.iter().nth(MAX_COVERED_PAGES).copied(),
    ]
    .into_iter()
    .flatten()
    .min();
    let keep = |list: &[u32]| {
        list.iter()
            .copied()
            .filter(|page| limit.is_none_or(|boundary| *page < boundary))
            .collect::<Vec<_>>()
    };
    let [recognition, extracted, empty] = pages.map(|list| keep(list));
    let visuals = document
        .visuals
        .iter()
        .filter(|visual| limit.is_none_or(|boundary| visual.page() < boundary))
        .collect::<Vec<_>>();
    // Two different facts, two different warnings: the parser's position data never guarantees
    // every inline or vector picture is found, and this record stops where its budget does.
    let mut warnings = vec!["pdf_visual_coverage_limited".to_owned()];
    if visuals.len() < document.visuals.len() {
        warnings.push("pdf_visual_occurrence_coverage_limited".to_owned());
    }
    if recognition.len() + extracted.len() + empty.len() < claimed.len() {
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
