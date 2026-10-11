const MAX_PREVIEW_UTF16: usize = 120_000;
const MAX_EXTRACTED_UTF16: usize = 4 * 1024 * 1024;

pub(crate) fn bounded_preview(text: &str) -> Option<String> {
    bounded_utf16(text, MAX_PREVIEW_UTF16)
}

pub(crate) fn bounded_extracted(text: &str) -> Option<String> {
    bounded_utf16(text, MAX_EXTRACTED_UTF16)
}

pub(crate) fn bounded_utf16_text(value: &str, limit: usize) -> String {
    crate::foundation::utf16::truncate_with_suffix(value, limit, "")
}

fn bounded_utf16(text: &str, limit: usize) -> Option<String> {
    let text = text.trim();
    (!text.is_empty()).then(|| crate::foundation::utf16::truncate_with_suffix(text, limit, "…"))
}
