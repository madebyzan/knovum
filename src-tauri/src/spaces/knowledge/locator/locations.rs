use serde_json::{Map, Value};

use super::{
    values::{
        ensure_keys, optional_nonnegative_i64, optional_string_limited, required_f64,
        required_nonnegative_i64, required_positive_i64, required_string, required_string_array,
        required_string_limited,
    },
    MAX_LOCATION_TEXT_CHARS, MAX_PDF_RECTS,
};

pub(super) fn validate_location(value: &Value) -> Result<(), String> {
    let object = value
        .as_object()
        .ok_or_else(|| "Knowledge locator location must be an object".to_string())?;
    let kind = required_string(object, "kind", "Knowledge locator location kind")?;
    match kind {
        "node" => ensure_keys(object, &["kind"], "Node locator"),
        "media" => validate_media_location(object),
        "pdf" => validate_pdf_location(object),
        "word" => validate_word_location(object),
        "slides" => validate_slides_location(object),
        "sheet" => validate_sheet_location(object),
        "markdown" => validate_text_location(object, "heading", "Markdown locator"),
        "note" => validate_text_location(object, "blockId", "Note locator"),
        "website" => validate_text_location(object, "section", "Website locator"),
        "code" => validate_code_location(object),
        _ => Err(format!("Unsupported knowledge locator kind '{kind}'")),
    }
}

fn validate_media_location(object: &Map<String, Value>) -> Result<(), String> {
    ensure_keys(object, &["kind", "startMs", "endMs"], "Media locator")?;
    let start = required_nonnegative_i64(object, "startMs", "Media locator startMs")?;
    let end = required_positive_i64(object, "endMs", "Media locator endMs")?;
    if end <= start {
        return Err("Media locator endMs must be greater than startMs".into());
    }
    Ok(())
}

fn validate_pdf_location(object: &Map<String, Value>) -> Result<(), String> {
    ensure_keys(object, &["kind", "page", "rects"], "PDF locator")?;
    required_positive_i64(object, "page", "PDF locator page")?;
    if let Some(rects) = object.get("rects") {
        let values = rects
            .as_array()
            .ok_or_else(|| "PDF locator rects must be an array".to_string())?;
        if values.len() > MAX_PDF_RECTS {
            return Err(format!("PDF locator rects exceeds {MAX_PDF_RECTS} entries"));
        }
        for rect in values {
            validate_rect(rect)?;
        }
    }
    Ok(())
}

fn validate_word_location(object: &Map<String, Value>) -> Result<(), String> {
    ensure_keys(
        object,
        &["kind", "section", "element", "headingPath", "part"],
        "Word locator",
    )?;
    required_nonnegative_i64(object, "section", "Word locator section")?;
    required_nonnegative_i64(object, "element", "Word locator element")?;
    required_string_array(object, "headingPath", "Word locator headingPath")?;
    if let Some(part) = object.get("part") {
        if !matches!(
            part.as_str(),
            Some(
                "header"
                    | "footer"
                    | "first_page_header"
                    | "first_page_footer"
                    | "even_page_header"
                    | "even_page_footer"
            )
        ) {
            return Err("Invalid Word content part".into());
        }
    }
    Ok(())
}

fn validate_slides_location(object: &Map<String, Value>) -> Result<(), String> {
    ensure_keys(object, &["kind", "slide", "element"], "Slides locator")?;
    required_positive_i64(object, "slide", "Slides locator slide")?;
    optional_nonnegative_i64(object, "element", "Slides locator element")?;
    Ok(())
}

fn validate_sheet_location(object: &Map<String, Value>) -> Result<(), String> {
    ensure_keys(
        object,
        &["kind", "sheet", "range", "element", "drawing"],
        "Sheet locator",
    )?;
    required_string_limited(
        object,
        "sheet",
        "Sheet locator sheet",
        MAX_LOCATION_TEXT_CHARS,
    )?;
    optional_string_limited(
        object,
        "range",
        "Sheet locator range",
        MAX_LOCATION_TEXT_CHARS,
    )?;
    optional_nonnegative_i64(object, "element", "Sheet locator element")?;
    if let Some(drawing) = object.get("drawing") {
        super::office_geometry::validate_drawing(drawing)?;
    }
    Ok(())
}

fn validate_text_location(
    object: &Map<String, Value>,
    text_key: &str,
    label: &str,
) -> Result<(), String> {
    ensure_keys(
        object,
        &["kind", text_key, "startOffset", "endOffset"],
        label,
    )?;
    optional_string_limited(
        object,
        text_key,
        &format!("{label} {text_key}"),
        MAX_LOCATION_TEXT_CHARS,
    )?;
    validate_optional_range(object, "startOffset", "endOffset", label)
}

fn validate_code_location(object: &Map<String, Value>) -> Result<(), String> {
    ensure_keys(
        object,
        &["kind", "path", "lineStart", "lineEnd"],
        "Code locator",
    )?;
    optional_string_limited(object, "path", "Code locator path", MAX_LOCATION_TEXT_CHARS)?;
    let start = required_positive_i64(object, "lineStart", "Code locator lineStart")?;
    let end = required_positive_i64(object, "lineEnd", "Code locator lineEnd")?;
    if end < start {
        return Err("Code locator lineEnd must be greater than or equal to lineStart".into());
    }
    Ok(())
}

fn validate_rect(value: &Value) -> Result<(), String> {
    let object = value
        .as_object()
        .ok_or_else(|| "PDF locator rect must be an object".to_string())?;
    ensure_keys(object, &["x", "y", "width", "height"], "PDF locator rect")?;
    required_f64(object, "x", "PDF locator rect x")?;
    required_f64(object, "y", "PDF locator rect y")?;
    if required_f64(object, "width", "PDF locator rect width")? <= 0.0
        || required_f64(object, "height", "PDF locator rect height")? <= 0.0
    {
        return Err("PDF locator rect width and height must be positive".into());
    }
    Ok(())
}

fn validate_optional_range(
    object: &Map<String, Value>,
    start_key: &str,
    end_key: &str,
    label: &str,
) -> Result<(), String> {
    let start = optional_nonnegative_i64(object, start_key, &format!("{label} {start_key}"))?;
    let end = optional_nonnegative_i64(object, end_key, &format!("{label} {end_key}"))?;
    if let (Some(start), Some(end)) = (start, end) {
        if end < start {
            return Err(format!(
                "{label} {end_key} must be greater than or equal to {start_key}"
            ));
        }
    }
    Ok(())
}
