use office_oxide::DocumentFormat;
use serde_json::json;

pub(super) fn sheet_drawing(
    sheet: &str,
    index: usize,
    x: i64,
    y: i64,
    width: i64,
    height: i64,
) -> serde_json::Value {
    let mut location = json!({"kind":"sheet", "sheet":sheet, "element":index});
    if width > 0 && height > 0 {
        location["drawing"] = json!({"xEmu":x,"yEmu":y,"widthEmu":width,"heightEmu":height});
    }
    location
}

pub(super) fn location(
    format: DocumentFormat,
    section: usize,
    element: usize,
    title: Option<&str>,
    headings: &[String],
    part: &str,
) -> serde_json::Value {
    if is_word(format) {
        let mut location = json!({ "kind": "word", "section": section, "element": element, "headingPath": headings });
        if part != "body" {
            location["part"] = json!(part);
        }
        return location;
    }
    if matches!(format, DocumentFormat::Ppt | DocumentFormat::Pptx) {
        return json!({ "kind": "slides", "slide": section + 1, "element": element });
    }
    match title.filter(|value| !value.trim().is_empty()) {
        Some(title) => json!({ "kind": "sheet", "sheet": title, "element": element }),
        None => json!({ "kind": "node" }),
    }
}

pub(super) fn is_word(format: DocumentFormat) -> bool {
    matches!(format, DocumentFormat::Doc | DocumentFormat::Docx)
}

pub(super) fn update_heading_path(path: &mut Vec<String>, level: u8, text: &str) {
    let depth = usize::from(level.clamp(1, 6));
    path.truncate(depth.saturating_sub(1));
    while path.len() < depth.saturating_sub(1) {
        path.push(String::new());
    }
    path.push(crate::utf16::truncate_with_suffix(text, 512, "…"));
    path.retain(|value| !value.is_empty());
}
