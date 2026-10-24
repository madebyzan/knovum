use super::covered;
use crate::documents::{
    extraction::{pdf::PdfDocumentContent, pdf_visuals::PdfVisual},
    pdf_input::{PdfInput, PdfRecognitionReason},
};
use crate::spaces::knowledge::model::document_coverage::{MAX_COVERED_PAGES, MAX_COVERED_VISUALS};

#[test]
fn a_document_longer_than_one_record_is_described_up_to_one_page() {
    // Two pictures a page, so the visual budget runs out first and the page lists have to stop
    // with it: a page claiming recognition and the input that recognises it are the same fact
    // stated twice, and a record that kept one without the other does not validate.
    let pages = MAX_COVERED_PAGES as u32 + 500;
    let document = document(
        pages,
        (1..=pages).collect(),
        (1..=pages)
            .flat_map(|page| [recognition(page), recognition(page)])
            .collect(),
        true,
    );

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
    let mut document = document(3, vec![2], vec![recognition(2)], false);
    document.pages_with_extracted_text = vec![1, 3];

    let covered = covered(&document);

    assert_eq!(covered.visuals.len(), 1);
    assert_eq!(covered.recognition, vec![2]);
    assert_eq!(covered.extracted, vec![1, 3]);
    // The parser caveat stands on its own: it is about what position data can find, not a budget.
    assert_eq!(covered.warnings, ["pdf_visual_coverage_limited"]);
}

fn document(
    page_count: u32,
    pages_requiring_recognition: Vec<u32>,
    visuals: Vec<PdfVisual>,
    text_truncated: bool,
) -> PdfDocumentContent {
    PdfDocumentContent {
        preview_text: None,
        extracted_text: None,
        segments: Vec::new(),
        page_count: page_count as usize,
        pages_requiring_recognition,
        pages_with_extracted_text: Vec::new(),
        pages_without_extractable_content: Vec::new(),
        visuals,
        text_truncated,
    }
}

fn recognition(page: u32) -> PdfVisual {
    PdfVisual {
        input: PdfInput::Page {
            page,
            reason: PdfRecognitionReason::Scanned,
        },
        context: String::new(),
    }
}
