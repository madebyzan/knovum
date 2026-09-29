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
        ],
        "imageEdit",
    )?;
    if edit.get("version").and_then(Value::as_u64) != Some(6) {
        return Err("imageEdit.version must be 6".into());
    }
    let crop = object_field(edit, "crop")?;
    reject_unknown(crop, ["x", "y", "width", "height"], "imageEdit.crop")?;
    let x = number(crop, "x", 0.0, 1.0, "imageEdit.crop")?;
    let y = number(crop, "y", 0.0, 1.0, "imageEdit.crop")?;
    let width = number(crop, "width", 0.0, 1.0, "imageEdit.crop")?;
    let height = number(crop, "height", 0.0, 1.0, "imageEdit.crop")?;
    if width == 0.0 || height == 0.0 || x + width > 1.0 || y + height > 1.0 {
        return Err("imageEdit.crop exceeds the image bounds".into());
    }
    let rotation = number(edit, "rotation", 0.0, 360.0, "imageEdit")?;
    if rotation >= 360.0 {
        return Err("imageEdit.rotation is invalid".into());
    }
    for key in ["flipHorizontal", "flipVertical"] {
        if edit.get(key).and_then(Value::as_bool).is_none() {
            return Err(format!("imageEdit.{key} must be a boolean"));
        }
    }
    let adjustments = object_field(edit, "adjustments")?;
    const ADJUSTMENT_KEYS: [&str; 16] = [
        "exposure",
        "contrast",
        "highlights",
        "shadows",
        "whites",
        "blacks",
        "saturation",
        "warmth",
        "sharpness",
        "vignette",
        "vibrance",
        "tint",
        "clarity",
        "grain",
        "softLight",
        "glow",
    ];
    reject_unknown(adjustments, ADJUSTMENT_KEYS, "imageEdit.adjustments")?;
    for key in ADJUSTMENT_KEYS {
        number(
            adjustments,
            key,
            if ["sharpness", "clarity", "grain", "softLight", "glow"].contains(&key) {
                0.0
            } else {
                -100.0
            },
            100.0,
            "imageEdit.adjustments",
        )?;
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

fn number(
    parent: &Map<String, Value>,
    key: &str,
    min: f64,
    max: f64,
    label: &str,
) -> Result<f64, String> {
    let value = parent
        .get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| format!("{label}.{key} must be a finite number"))?;
    if !value.is_finite() || value < min || value > max {
        return Err(format!("{label}.{key} must be between {min} and {max}"));
    }
    Ok(value)
}
