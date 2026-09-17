use super::model::{DocumentCoverage, DocumentParser, VisualState};
use crate::documents::pdf_input::PdfInput;
use std::collections::BTreeSet;

pub(super) fn validate(report: &DocumentCoverage) -> Result<(), String> {
    if report.schema_version != 1
        || report.source_revision.is_empty()
        || report.source_revision.len() > 128
        || report
            .page_count
            .is_some_and(|count| count == 0 || count > 2000)
        || report.resources.len() > 2000
        || report.occurrences.len() > 2000
    {
        return Err("Document coverage exceeds its contract".into());
    }
    if (report.parser == DocumentParser::PdfInspector) != report.page_count.is_some() {
        return Err("Document parser and page coverage disagree".into());
    }
    validate_pages(report)?;
    if report.warnings.len() > 16
        || report.warnings.iter().any(|warning| {
            !matches!(
                warning.as_str(),
                "office_vector_drawings_unsupported"
                    | "spreadsheet_image_cell_anchors_unavailable"
                    | "spreadsheet_chart_rendering_unsupported"
                    | "legacy_office_visual_occurrence_coverage_limited"
                    | "legacy_spreadsheet_formulas_unavailable"
                    | "pdf_visual_coverage_limited"
            )
        })
    {
        return Err("Invalid document coverage warning".into());
    }
    Ok(())
}

fn validate_pages(report: &DocumentCoverage) -> Result<(), String> {
    let count = report.page_count.unwrap_or(0) as u32;
    let mut claimed = BTreeSet::new();
    for pages in [
        &report.pages_requiring_recognition,
        &report.pages_with_extracted_text,
        &report.pages_without_extractable_content,
    ] {
        for page in pages {
            if *page == 0 || *page > count || !claimed.insert(*page) {
                return Err(
                    "PDF coverage pages must be unique, in range and non-overlapping".into(),
                );
            }
        }
    }
    validate_recognition_inputs(report)
}

fn validate_recognition_inputs(report: &DocumentCoverage) -> Result<(), String> {
    let page_inputs = report
        .occurrences
        .iter()
        .filter_map(|occurrence| match &occurrence.pdf_input {
            Some(PdfInput::Page { page, .. }) => Some(*page),
            _ => None,
        })
        .collect::<Vec<_>>();
    if page_inputs.len() != report.pages_requiring_recognition.len()
        || page_inputs.iter().copied().collect::<BTreeSet<_>>()
            != report.pages_requiring_recognition.iter().copied().collect()
    {
        return Err("PDF recognition coverage and page inputs disagree".into());
    }
    for occurrence in &report.occurrences {
        if let Some(PdfInput::Image { page, .. }) = &occurrence.pdf_input {
            if report.pages_requiring_recognition.contains(page)
                || report.pages_without_extractable_content.contains(page)
            {
                return Err("PDF page and region inputs must not duplicate coverage".into());
            }
        }
        if occurrence.state == VisualState::Completed && occurrence.resource_id.is_none() {
            return Err("Completed document analysis requires its durable raster".into());
        }
    }
    Ok(())
}
