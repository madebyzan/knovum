use serde_json::Value;

/// The summary a captured page holds, as authoring content.
///
/// A capture is metadata and extracted prose rather than a file, so the engine assembles what it
/// holds: that summary is the capture's own format, and it is also what every document
/// serialization reads. Assembling it in one place is what keeps the saved Markdown and the saved
/// PDF the same document.
pub(super) fn summary(payload: &Value) -> Result<String, String> {
    let title = text(payload, "title").ok_or("The capture has no title")?;
    let address = text(payload, "url").ok_or("The capture has no address")?;
    let mut lines = vec![
        format!("# {title}"),
        String::new(),
        format!("Source: {address}"),
    ];
    if let Some(canonical) = text(payload, "canonicalUrl").filter(|value| *value != address) {
        lines.push(format!("Canonical: {canonical}"));
    }
    if let Some(site) = text(payload, "siteName") {
        lines.push(format!("Site: {site}"));
    }
    for field in ["description", "contentPreview"] {
        if let Some(value) = text(payload, field) {
            lines.extend([String::new(), value.to_owned()]);
        }
    }
    let sections = payload
        .get("websiteSections")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    for section in sections {
        if let Some(heading) = text(section, "heading") {
            lines.extend([String::new(), format!("## {heading}")]);
        }
        let body = text(section, "text").ok_or("A captured section has no text")?;
        lines.extend([String::new(), body.to_owned()]);
    }
    Ok(format!("{}\n", lines.join("\n")))
}

fn text<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}
