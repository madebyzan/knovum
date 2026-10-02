use std::collections::BTreeSet;

use super::{
    DocumentAnalysis, DocumentCoverage, DocumentOccurrence, DocumentResource, VisualState,
    MAX_RESOURCE_BYTES, MAX_RESOURCE_DIMENSION, MAX_RESOURCE_PIXELS, MAX_TOTAL_RESOURCE_BYTES,
};

pub(super) fn validate(report: &DocumentCoverage) -> Result<(), String> {
    super::coverage::validate(report)?;
    let mut resource_ids = BTreeSet::new();
    let mut bytes = 0_u64;
    for resource in &report.resources {
        validate_resource(resource)?;
        if !resource_ids.insert(resource.id.as_str()) {
            return Err("Document resource IDs must be unique".into());
        }
        bytes += resource.byte_size;
    }
    if bytes > MAX_TOTAL_RESOURCE_BYTES {
        return Err("Document visual resources exceed 256 MiB".into());
    }
    let mut occurrence_ids = BTreeSet::new();
    let mut referenced_resource_ids = BTreeSet::new();
    let mut text_width = 0_usize;
    for occurrence in &report.occurrences {
        validate_occurrence(occurrence, report, &resource_ids)?;
        if let Some(resource_id) = occurrence.resource_id.as_deref() {
            referenced_resource_ids.insert(resource_id);
        }
        if !occurrence_ids.insert(occurrence.id.as_str()) {
            return Err("Document occurrence IDs must be unique".into());
        }
        if let Some(analysis) = &occurrence.analysis {
            validate_analysis(analysis)?;
            text_width += analysis.description.encode_utf16().count()
                + analysis.detected_text.encode_utf16().count();
        }
    }
    if resource_ids != referenced_resource_ids {
        return Err("Document resources must belong to at least one occurrence".into());
    }
    if text_width > 4 * 1024 * 1024 {
        return Err("Document visual text exceeds 4 Mi UTF-16 units".into());
    }
    Ok(())
}

fn validate_resource(resource: &DocumentResource) -> Result<(), String> {
    if resource.id.len() != 64
        || !resource
            .id
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        || resource.path.is_empty()
        || resource.path.encode_utf16().count() > 4096
        || resource.media_type != "image/png"
        || resource.width == 0
        || resource.height == 0
        || resource.width > MAX_RESOURCE_DIMENSION
        || resource.height > MAX_RESOURCE_DIMENSION
        || u64::from(resource.width) * u64::from(resource.height) > MAX_RESOURCE_PIXELS
        || resource.byte_size == 0
        || resource.byte_size > MAX_RESOURCE_BYTES
    {
        return Err("Invalid document raster resource".into());
    }
    Ok(())
}

fn validate_occurrence(
    occurrence: &DocumentOccurrence,
    report: &DocumentCoverage,
    resources: &BTreeSet<&str>,
) -> Result<(), String> {
    if occurrence.id.is_empty()
        || occurrence.id.encode_utf16().count() > 1024
        || occurrence.context.encode_utf16().count() > 4000
        || occurrence
            .alt_text
            .as_ref()
            .is_some_and(|text| text.encode_utf16().count() > 16_000)
        || occurrence
            .error
            .as_ref()
            .is_some_and(|text| text.encode_utf16().count() > 16_000)
        || occurrence
            .resource_id
            .as_ref()
            .is_some_and(|id| !resources.contains(id.as_str()))
        || (occurrence.state == VisualState::Completed) != occurrence.analysis.is_some()
    {
        return Err("Invalid document visual occurrence".into());
    }
    validate_input(occurrence, report)?;
    crate::spaces::knowledge::locator::validate(
        &serde_json::json!({"sourceRevision":report.source_revision,"location":occurrence.location}),
        Some(&report.source_revision),
    )
}

fn validate_input(
    occurrence: &DocumentOccurrence,
    report: &DocumentCoverage,
) -> Result<(), String> {
    use crate::documents::pdf_input::PdfInput;
    match (&occurrence.pdf_input, report.page_count) {
        (Some(input), Some(count)) => {
            if input.page() == 0
                || input.page() > count as u32
                || occurrence.location != input.location()
            {
                return Err("PDF visual input and location disagree".into());
            }
            match input {
                PdfInput::Image { rect, .. } => rect.validate()?,
                PdfInput::Page { page, .. } => {
                    if !report.pages_requiring_recognition.contains(page) {
                        return Err("PDF page has no recognition requirement".into());
                    }
                }
            }
        }
        (None, None) => {
            if occurrence.resource_id.is_none() && occurrence.state != VisualState::Unsupported {
                return Err("Office visual occurrence requires a raster resource".into());
            }
        }
        _ => return Err("Document visual input belongs to a different format".into()),
    }
    Ok(())
}

pub(crate) fn validate_analysis(analysis: &DocumentAnalysis) -> Result<(), String> {
    analysis.execution_source.validate()?;
    if analysis.analysis_version != "document-visual:1"
        || analysis.generated_at < 0
        || analysis.description.encode_utf16().count() > 48_000
        || analysis.detected_text.encode_utf16().count() > 48_000
    {
        return Err("Invalid document visual analysis".into());
    }
    Ok(())
}
