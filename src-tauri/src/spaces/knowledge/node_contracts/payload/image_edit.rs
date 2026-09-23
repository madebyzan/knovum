use serde_json::{Map, Value};

use super::super::validation::reject_unknown;

pub(super) fn validate(value: Option<&Value>) -> Result<(), String> {
    let Some(value) = value else {
        return Ok(());
    };
    let edit = object(value, "imageEdit")?;
    reject_unknown(
        edit,
        [
            "version",
            "crop",
            "rotation",
            "flipHorizontal",
            "flipVertical",
            "adjustments",
            "eraserStrokes",
        ],
        "imageEdit",
    )?;
    if number(edit, "version", 1.0, 1.0)? != 1.0 {
        return Err("imageEdit.version must be 1".into());
    }
    let crop = object_field(edit, "crop")?;
    reject_unknown(crop, ["x", "y", "width", "height"], "imageEdit.crop")?;
    let x = number(crop, "x", 0.0, 1.0)?;
    let y = number(crop, "y", 0.0, 1.0)?;
    let width = number(crop, "width", 0.0, 1.0)?;
    let height = number(crop, "height", 0.0, 1.0)?;
    if width == 0.0 || height == 0.0 || x + width > 1.0 || y + height > 1.0 {
        return Err("imageEdit.crop exceeds the image bounds".into());
    }
    let rotation = number(edit, "rotation", 0.0, 270.0)?;
    if ![0.0, 90.0, 180.0, 270.0].contains(&rotation) {
        return Err("imageEdit.rotation is invalid".into());
    }
    for key in ["flipHorizontal", "flipVertical"] {
        if edit.get(key).and_then(Value::as_bool).is_none() {
            return Err(format!("imageEdit.{key} must be a boolean"));
        }
    }
    let adjustments = object_field(edit, "adjustments")?;
    reject_unknown(
        adjustments,
        ["brightness", "contrast", "saturation"],
        "imageEdit.adjustments",
    )?;
    for key in ["brightness", "contrast", "saturation"] {
        number(adjustments, key, -100.0, 100.0)?;
    }
    let strokes = edit
        .get("eraserStrokes")
        .and_then(Value::as_array)
        .ok_or("imageEdit.eraserStrokes must be an array")?;
    if strokes.len() > 500 {
        return Err("imageEdit.eraserStrokes exceeds 500 entries".into());
    }
    for stroke in strokes {
        let stroke = object(stroke, "imageEdit.eraserStroke")?;
        reject_unknown(stroke, ["diameter", "points"], "imageEdit.eraserStroke")?;
        if number(stroke, "diameter", 0.0, 0.5)? == 0.0 {
            return Err("imageEdit.eraserStroke.diameter must be positive".into());
        }
        let points = stroke
            .get("points")
            .and_then(Value::as_array)
            .ok_or("imageEdit.eraserStroke.points must be an array")?;
        if points.len() < 4 || points.len() > 4096 || points.len() % 2 != 0 {
            return Err("imageEdit.eraserStroke.points length is invalid".into());
        }
        for point in points {
            if !point
                .as_f64()
                .is_some_and(|value| (0.0..=1.0).contains(&value))
            {
                return Err("imageEdit.eraserStroke.points must be normalized numbers".into());
            }
        }
    }
    Ok(())
}

fn object<'a>(value: &'a Value, label: &str) -> Result<&'a Map<String, Value>, String> {
    value
        .as_object()
        .ok_or_else(|| format!("{label} must be an object"))
}

fn object_field<'a>(
    parent: &'a Map<String, Value>,
    key: &str,
) -> Result<&'a Map<String, Value>, String> {
    object(
        parent
            .get(key)
            .ok_or_else(|| format!("imageEdit.{key} is required"))?,
        &format!("imageEdit.{key}"),
    )
}

fn number(parent: &Map<String, Value>, key: &str, min: f64, max: f64) -> Result<f64, String> {
    let value = parent
        .get(key)
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite() && *value >= min && *value <= max)
        .ok_or_else(|| format!("imageEdit.{key} is outside the allowed range"))?;
    Ok(value)
}
