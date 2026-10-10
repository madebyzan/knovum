pub(super) fn extract_markdown(bytes: &[u8]) -> Result<String, String> {
    let document = pdf_inspector::extract_pages_markdown_mem(bytes, None)
        .map_err(|error| format!("PDF conversion parsing failed: {error}"))?;
    if document.pages.is_empty() || document.pages.len() > 2_000 {
        return Err("PDF conversion requires between 1 and 2000 pages".into());
    }
    let mut markdown = String::new();
    for page in document.pages {
        if page.needs_ocr {
            return Err(format!(
                "PDF page {} requires recognition; direct conversion is unavailable",
                page.page + 1
            ));
        }
        if markdown.len() + page.markdown.len() + 2 > 4 * 1024 * 1024 {
            return Err("PDF conversion text exceeds 4 MiB".into());
        }
        if !markdown.is_empty() {
            markdown.push_str("\n\n");
        }
        markdown.push_str(&page.markdown);
    }
    if markdown.trim().is_empty() {
        return Err("PDF has no directly convertible text".into());
    }
    Ok(markdown)
}
