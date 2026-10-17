use serde_json::Value;

use crate::{
    documents::{generation::input::RecognizedPage, pdf_input::PdfInput},
    spaces::knowledge::model::document_coverage::{DocumentCoverage, VisualState},
};

pub(super) fn load(content: &Value) -> Result<Vec<RecognizedPage>, String> {
    if content.get("documentCoverage").is_none() {
        return Ok(vec![]);
    }
    let coverage = DocumentCoverage::from_node_content(content)?;
    coverage.pages_requiring_recognition.iter().map(|page| {
        let occurrence = coverage.occurrences.iter().find(|occurrence| {
            matches!(occurrence.pdf_input, Some(PdfInput::Page { page: number, .. }) if number == *page)
        }).ok_or("Recognize all scanned PDF pages before conversion")?;
        if occurrence.state != VisualState::Completed { return Err("Recognize all scanned PDF pages before conversion".into()); }
        let text = occurrence.analysis.as_ref().map(|analysis| analysis.detected_text.trim())
            .filter(|text| !text.is_empty()).ok_or("PDF recognition has no convertible text")?;
        Ok(RecognizedPage { page: *page, text: text.to_owned() })
    }).collect()
}
