mod budget;
mod content;
mod document_text;
mod images;
mod location;
mod segments;
mod spreadsheet;
mod spreadsheet_visuals;
mod tables;
mod visuals;

use std::path::Path;

use office_oxide::{Document, DocumentFormat};
use tokio_util::sync::CancellationToken;

use super::DocumentSegment;

pub(crate) struct OfficeDocument {
    pub(crate) preview_text: Option<String>,
    pub(crate) extracted_text: Option<String>,
    pub(crate) segments: Vec<DocumentSegment>,
    pub(crate) images: Vec<super::visual::DocumentImage>,
    pub(crate) warnings: Vec<String>,
}

pub(crate) fn supports(source: &Path) -> bool {
    matches!(
        DocumentFormat::from_path(source),
        Some(
            DocumentFormat::Doc
                | DocumentFormat::Docx
                | DocumentFormat::Xls
                | DocumentFormat::Xlsx
                | DocumentFormat::Ppt
                | DocumentFormat::Pptx
        )
    )
}

pub(crate) fn extract(
    source: &Path,
    cancellation: &CancellationToken,
) -> Result<OfficeDocument, String> {
    ensure_active(cancellation)?;
    let _permit = super::acquire_parse_permit(cancellation)?;
    budget::preflight(source)?;
    let document =
        Document::open(source).map_err(|error| format!("Office parsing failed: {error}"))?;
    ensure_active(cancellation)?;
    if document
        .as_xls()
        .is_some_and(|workbook| workbook.sheets.is_empty())
    {
        return Err(
            "Legacy spreadsheet has no readable worksheets; it may be encrypted or unsupported"
                .into(),
        );
    }
    let format = DocumentFormat::from_path(source).ok_or("Unsupported Office format")?;
    let mut text_budget = budget::TextBudget::default();
    let (segments, mut images) = if let Some(workbook) = document.as_xlsx() {
        (
            spreadsheet::segments(workbook, cancellation, &mut text_budget)?,
            spreadsheet_visuals::collect(workbook, cancellation)?,
        )
    } else if let Some(workbook) = document.as_xls() {
        (
            spreadsheet::legacy_segments(workbook, cancellation, &mut text_budget)?,
            images::DocumentImages::default(),
        )
    } else {
        let ir = document.to_ir();
        let images = visuals::collect(&ir, format, cancellation)?;
        let segments = segments::extract(&ir, format, cancellation, &mut text_budget)?;
        (segments, images)
    };
    images::collect_legacy(&document, &mut images)?;
    let images = images.into_entries();
    let plain = segments
        .iter()
        .map(|segment| segment.text.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");
    let mut warnings = vec!["office_vector_drawings_unsupported".into()];
    if matches!(format, DocumentFormat::Xls | DocumentFormat::Xlsx) {
        if format == DocumentFormat::Xlsx && !images.is_empty() {
            warnings.push("spreadsheet_image_cell_anchors_unavailable".into());
        }
        if document
            .as_xlsx()
            .is_some_and(|workbook| !workbook.chart_text.is_empty())
        {
            warnings.push("spreadsheet_chart_rendering_unsupported".into());
        }
    }
    if matches!(
        format,
        DocumentFormat::Doc | DocumentFormat::Xls | DocumentFormat::Ppt
    ) {
        warnings.push("legacy_office_visual_occurrence_coverage_limited".into());
    }
    if format == DocumentFormat::Xls {
        warnings.push("legacy_spreadsheet_formulas_unavailable".into());
    }
    Ok(OfficeDocument {
        preview_text: super::text::bounded_preview(&plain),
        extracted_text: super::text::bounded_extracted(&plain),
        segments,
        images,
        warnings,
    })
}

fn ensure_active(cancellation: &CancellationToken) -> Result<(), String> {
    if cancellation.is_cancelled() {
        Err("Document processing cancelled".into())
    } else {
        Ok(())
    }
}
