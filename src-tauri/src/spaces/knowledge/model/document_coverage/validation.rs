use std::collections::BTreeSet;

use super::{
    DocumentCoverage, DocumentOccurrence, DocumentResource, MAX_RESOURCE_BYTES,
    MAX_RESOURCE_DIMENSION, MAX_RESOURCE_PIXELS, MAX_TOTAL_RESOURCE_BYTES,
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
    for occurrence in &report.occurrences {
        validate_occurrence(occurrence, report, &resource_ids)?;
        if let Some(resource_id) = occurrence.resource_id.as_deref() {
            referenced_resource_ids.insert(resource_id);
        }
        if !occurrence_ids.insert(occurrence.id.as_str()) {
            return Err("Document occurrence IDs must be unique".into());
        }
    }
    if resource_ids != referenced_resource_ids {
        return Err("Document resources must belong to at least one occurrence".into());
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
            .unavailable
            .as_ref()
            .is_some_and(|text| text.is_empty() || text.encode_utf16().count() > 16_000)
        || occurrence
            .resource_id
            .as_ref()
            .is_some_and(|id| !resources.contains(id.as_str()))
        // Every picture is either extracted or says why it is not, never both or neither.
        || occurrence.resource_id.is_some() == occurrence.unavailable.is_some()
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
            // A PDF region is rendered from the page itself, so it always has a raster.
            if occurrence.resource_id.is_none() {
                return Err("PDF visual input requires its rendered raster".into());
            }
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
                        return Err("A whole-page picture belongs to a scanned page".into());
                    }
                }
            }
        }
        (None, None) => {}
        _ => return Err("Document visual input belongs to a different format".into()),
    }
    Ok(())
}
