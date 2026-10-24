use super::{document::prepare::covered, processing_error, ASSET_PROCESSING_CANCELLED};
use crate::documents::{
    extraction::{pdf::PdfDocumentContent, pdf_visuals::PdfVisual},
    pdf_input::PdfInput,
};
use crate::spaces::knowledge::model::document_coverage::{MAX_COVERED_PAGES, MAX_COVERED_VISUALS};

#[test]
fn asset_processing_cancellation_has_an_explicit_product_code() {
    let error = processing_error(ASSET_PROCESSING_CANCELLED.to_owned());
    assert_eq!(error.code, "space_asset_processing_cancelled");
    assert_eq!(
        processing_error("enrichment-error: vendor detail".into()).code,
        "operation_failed"
    );
}

#[test]
fn a_document_longer_than_one_record_is_described_up_to_one_page() {
    // Two pictures a page, so the visual budget runs out first. Everything the record keeps has to
    // agree: a page claiming recognition and the input that recognises it are the same fact, and
    // `DocumentCoverage::validate` rejects a record that kept one and dropped the other.
    let pages = MAX_COVERED_PAGES as u32 + 500;
    let document = PdfDocumentContent {
        preview_text: None,
        extracted_text: None,
        segments: Vec::new(),
        page_count: pages as usize,
        pages_requiring_recognition: (1..=pages).collect(),
        pages_with_extracted_text: Vec::new(),
        pages_without_extractable_content: Vec::new(),
        visuals: (1..=pages)
            .flat_map(|page| [recognition(page), recognition(page)])
            .collect(),
        text_truncated: true,
    };

    let covered = covered(&document);

    let boundary = MAX_COVERED_VISUALS as u32 / 2 + 1;
    assert_eq!(covered.visuals.len(), (boundary as usize - 1) * 2);
    assert_eq!(covered.recognition, (1..boundary).collect::<Vec<_>>());
    assert!(covered
        .visuals
        .iter()
        .all(|visual| visual.page() < boundary));
    assert_eq!(
        covered.warnings,
        [
            "pdf_visual_coverage_limited",
            "pdf_visual_occurrence_coverage_limited",
            "pdf_page_coverage_limited",
            "pdf_text_coverage_limited",
        ]
    );
}

#[test]
fn a_document_that_fits_is_described_whole_and_claims_no_limit() {
    let document = PdfDocumentContent {
        preview_text: None,
        extracted_text: None,
        segments: Vec::new(),
        page_count: 3,
        pages_requiring_recognition: vec![2],
        pages_with_extracted_text: vec![1, 3],
        pages_without_extractable_content: Vec::new(),
        visuals: vec![recognition(2)],
        text_truncated: false,
    };

    let covered = covered(&document);

    assert_eq!(covered.visuals.len(), 1);
    assert_eq!(covered.recognition, vec![2]);
    assert_eq!(covered.extracted, vec![1, 3]);
    // The parser caveat stands on its own: it is about what position data can find, not a budget.
    assert_eq!(covered.warnings, ["pdf_visual_coverage_limited"]);
}

fn recognition(page: u32) -> PdfVisual {
    PdfVisual {
        input: PdfInput::Page {
            page,
            reason: crate::documents::pdf_input::PdfRecognitionReason::Scanned,
        },
        context: String::new(),
    }
}
