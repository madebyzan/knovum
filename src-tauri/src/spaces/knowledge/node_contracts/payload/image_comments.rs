use std::collections::HashSet;

use serde_json::Value;

use super::super::validation::reject_unknown;

pub(super) fn validate(value: Option<&Value>) -> Result<(), String> {
    let Some(value) = value else {
        return Ok(());
    };
    let comments = value.as_array().ok_or("Image comments must be an array")?;
    if comments.len() > 200 {
        return Err("Image comments exceed 200 entries".into());
    }
    let mut ids = HashSet::with_capacity(comments.len());
    for comment in comments {
        let object = comment
            .as_object()
            .ok_or("Image comment must be an object")?;
        reject_unknown(
            object,
            ["id", "x", "y", "shape", "text", "createdAt"],
            "Image comment",
        )?;
        let id = object
            .get("id")
            .and_then(Value::as_str)
            .ok_or("Image comment id is required")?;
        let id = uuid::Uuid::parse_str(id).map_err(|_| "Image comment id is invalid")?;
        if !ids.insert(id) {
            return Err("Image comment ids must be unique".into());
        }
        let text = object
            .get("text")
            .and_then(Value::as_str)
            .ok_or("Image comment text is required")?;
        if text.trim().is_empty() || text.chars().count() > 2000 {
            return Err("Image comment text is invalid".into());
        }
        for key in ["x", "y"] {
            coordinate(object.get(key), "Image comment position")?;
        }
        validate_shape(object.get("shape"))?;
        let created_at = object.get("createdAt").and_then(Value::as_u64);
        if created_at.is_none_or(|value| value > 9_007_199_254_740_991) {
            return Err("Image comment time is invalid".into());
        }
    }
    Ok(())
}

fn coordinate(value: Option<&Value>, label: &str) -> Result<f64, String> {
    let number = value
        .and_then(Value::as_f64)
        .ok_or_else(|| format!("{label} is required"))?;
    if !(0.0..=1.0).contains(&number) {
        return Err(format!("{label} is invalid"));
    }
    Ok(number)
}

fn validate_shape(value: Option<&Value>) -> Result<(), String> {
    let Some(value) = value else {
        return Ok(());
    };
    let shape = value
        .as_object()
        .ok_or("Image comment shape must be an object")?;
    match shape.get("kind").and_then(Value::as_str) {
        Some("stroke") => {
            reject_unknown(shape, ["kind", "points"], "Image comment stroke")?;
            let points = shape
                .get("points")
                .and_then(Value::as_array)
                .ok_or("Image comment stroke points are required")?;
            if points.len() < 4 || points.len() > 1024 || points.len() % 2 != 0 {
                return Err("Image comment stroke length is invalid".into());
            }
            for point in points {
                coordinate(Some(point), "Image comment stroke coordinate")?;
            }
        }
        Some("region") => {
            reject_unknown(
                shape,
                ["kind", "x", "y", "width", "height"],
                "Image comment region",
            )?;
            let x = coordinate(shape.get("x"), "Image comment region x")?;
            let y = coordinate(shape.get("y"), "Image comment region y")?;
            let width = coordinate(shape.get("width"), "Image comment region width")?;
            let height = coordinate(shape.get("height"), "Image comment region height")?;
            if width <= 0.0 || height <= 0.0 || x + width > 1.0 || y + height > 1.0 {
                return Err("Image comment region exceeds image bounds".into());
            }
        }
        _ => return Err("Image comment shape kind is invalid".into()),
    }
    Ok(())
}
