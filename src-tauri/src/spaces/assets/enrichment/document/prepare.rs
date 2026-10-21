use std::path::Path;

use tauri::AppHandle;

use super::resource;
use crate::documents::extraction::{pdf::PdfDocumentContent, visual::DocumentImage};
use crate::spaces::knowledge::model::document_coverage::{
    DocumentCoverage, DocumentOccurrence, DocumentParser, MAX_TOTAL_RESOURCE_BYTES,
};

pub(in crate::spaces::assets::enrichment) fn pdf(
    app: &AppHandle,
    source: &Path,
    document: &PdfDocumentContent,
    revision: &str,
    cancellation: &tokio_util::sync::CancellationToken,
    claim_managed_path: &mut dyn FnMut(&Path) -> Result<(), String>,
) -> Result<DocumentCoverage, String> {
    let mut report = DocumentCoverage {
        schema_version: 1,
        parser: DocumentParser::PdfInspector,
        source_revision: revision.into(),
        page_count: Some(document.page_count),
        pages_requiring_recognition: document.pages_requiring_recognition.clone(),
        pages_with_extracted_text: document.pages_with_extracted_text.clone(),
        pages_without_extractable_content: document.pages_without_extractable_content.clone(),
        resources: Vec::new(),
        occurrences: Vec::new(),
        warnings: vec!["pdf_visual_coverage_limited".into()],
    };
    for (index, visual) in document.visuals.iter().enumerate() {
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
