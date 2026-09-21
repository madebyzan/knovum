use serde_json::Value;

pub(super) fn validate_content_appearance(value: &Option<Value>) -> Result<(), String> {
    validate_appearance(value, false)
}

pub(super) fn validate_section_appearance(value: &Option<Value>) -> Result<(), String> {
    validate_appearance(value, true)
}

fn validate_appearance(value: &Option<Value>, section: bool) -> Result<(), String> {
    let Some(value) = value else { return Ok(()) };
    let fields = value
        .as_object()
        .ok_or("Structure appearance must be an object")?;
    if fields.contains_key("fill") && fields.contains_key("customFill") {
        return Err("Appearance cannot set both fill and customFill".into());
    }
    for (key, value) in fields {
        let valid = match key.as_str() {
            "fill" => one_of(value, &["yellow", "green", "blue", "pink", "gray"]),
            "customFill" => value.as_str().is_some_and(valid_hex_color),
            "background" => value.is_string(),
            "backgroundStyle" => one_of(value, &["cover", "ratio", "repeat"]),
            "borderStyle" if section => one_of(value, &["solid", "dashed"]),
            "textColor" if !section => {
                one_of(value, &["ink", "muted", "red", "blue", "green", "purple"])
            }
            "fontSize" if !section => number_between(value, 2.5, 576.0),
            "typographyScale" if !section => number_between(value, 0.25, 4.0),
            "fontWeight" if !section => one_of(value, &["regular", "bold"]),
            "fontStyle" if !section => one_of(value, &["normal", "italic"]),
            "textDecoration" if !section => one_of(value, &["none", "underline"]),
            "textAlign" if !section => one_of(value, &["left", "center", "right"]),
            "verticalAlign" if !section => one_of(value, &["top", "middle", "bottom"]),
            _ => false,
        };
        if !valid {
            return Err(format!("Invalid appearance field: {key}"));
        }
    }
    Ok(())
}

fn one_of(value: &Value, allowed: &[&str]) -> bool {
    value.as_str().is_some_and(|value| allowed.contains(&value))
}

fn number_between(value: &Value, minimum: f64, maximum: f64) -> bool {
    value
        .as_f64()
        .is_some_and(|number| number.is_finite() && (minimum..=maximum).contains(&number))
}

fn valid_hex_color(value: &str) -> bool {
    value.len() == 7
        && value.starts_with('#')
        && value.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
}
