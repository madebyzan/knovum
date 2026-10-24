use std::{io::Read, path::Path};

use serde_json::json;
use tokio_util::sync::CancellationToken;

use super::DocumentSegment;

const MAX_SOURCE_BYTES: u64 = 128 * 1024 * 1024;
const MAX_DOCUMENT_UTF16: usize = 4 * 1024 * 1024;

pub(crate) struct PdfDocumentContent {
    pub(crate) preview_text: Option<String>,
    pub(crate) extracted_text: Option<String>,
    pub(crate) segments: Vec<DocumentSegment>,
    pub(crate) page_count: usize,
    pub(crate) pages_requiring_recognition: Vec<u32>,
    pub(crate) pages_with_extracted_text: Vec<u32>,
    pub(crate) pages_without_extractable_content: Vec<u32>,
    pub(crate) visuals: Vec<super::pdf_visuals::PdfVisual>,
    /// Whether pages were left unread because the document's text passed the budget above. The
    /// pages that were read are in `pages_with_extracted_text`; the rest are in neither list.
    pub(crate) text_truncated: bool,
}

pub(crate) fn extract(
    source: &Path,
    cancellation: &CancellationToken,
) -> Result<PdfDocumentContent, String> {
    ensure_active(cancellation)?;
    let _permit = super::acquire_parse_permit(cancellation)?;
    let file = std::fs::File::open(source).map_err(|error| format!("PDF open failed: {error}"))?;
    let mut bytes = Vec::new();
    file.take(MAX_SOURCE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("PDF read failed: {error}"))?;
    if bytes.len() as u64 > MAX_SOURCE_BYTES {
        return Err("PDF exceeds the 128 MiB processing budget".into());
    }
    let extracted_document = pdf_inspector::extract_pages_markdown_mem(&bytes, None)
        .map_err(|error| format!("PDF parsing failed: {error}"))?;
    ensure_active(cancellation)?;
    if extracted_document.pages.is_empty() {
        return Err("PDF has no pages".into());
    }
    let mut text = String::new();
    let positions = pdf_inspector::extract_text_with_positions_mem_in_frame(
        &bytes,
        None,
        pdf_inspector::PositionFrame::Display,
    )
    .map_err(|error| format!("PDF visual discovery failed: {error}"))?;
    ensure_active(cancellation)?;
    let mut positioned_pages = std::collections::BTreeMap::<u32, Vec<_>>::new();
    for item in positions {
        positioned_pages.entry(item.page).or_default().push(item);
    }
    let mut segments = Vec::new();
    let page_count = extracted_document.pages.len();
    let mut visual_pages = Vec::new();
    let mut text_pages = Vec::new();
    let mut empty_pages = Vec::new();
    let mut visuals = Vec::new();
    let mut text_width = 0;
    let mut text_truncated = false;
    for page in extracted_document.pages {
        ensure_active(cancellation)?;
        let items = positioned_pages
            .remove(&(page.page + 1))
            .unwrap_or_default();
        let planned = super::pdf_visuals::plan(page.page + 1, page.ocr_reason.as_deref(), &items)?;
        if planned.iter().any(|visual| {
            matches!(
                visual.input,
                crate::documents::pdf_input::PdfInput::Page { .. }
            )
        }) {
            visual_pages.push(page.page + 1);
        }
        if page.markdown.trim().is_empty() && planned.is_empty() {
            empty_pages.push(page.page + 1);
        }
        visuals.extend(planned);
        if page.needs_ocr {
            continue;
        }
        // The text budget bounds what one node holds, so reading stops here and the pages that
        // were read say so. Discovery continues: a picture on page 3000 is still found.
        let width = page.markdown.encode_utf16().count() + 2;
        if text_width + width > MAX_DOCUMENT_UTF16 {
            text_truncated = true;
            continue;
        }
        text_width += width;
        if !text.is_empty() {
            text.push_str("\n\n");
        }
        text.push_str(&page.markdown);
        if !page.markdown.trim().is_empty() {
            text_pages.push(page.page + 1);
            segments.push(DocumentSegment {
                kind: "page",
                text: page.markdown,
                location: json!({ "kind": "pdf", "page": page.page + 1 }),
                metadata: json!({ "parser": "pdf-inspector:1.20.0", "page": page.page + 1 }),
            });
        }
    }
    Ok(PdfDocumentContent {
        preview_text: super::text::bounded_preview(&text),
        extracted_text: super::text::bounded_extracted(&text),
        segments,
        page_count,
        pages_requiring_recognition: visual_pages,
        pages_with_extracted_text: text_pages,
        pages_without_extractable_content: empty_pages,
        visuals,
        text_truncated,
    })
}

fn ensure_active(cancellation: &CancellationToken) -> Result<(), String> {
    if cancellation.is_cancelled() {
        Err("Document processing cancelled".into())
    } else {
        Ok(())
    }
}
