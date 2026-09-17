use super::values::{ensure_keys, required_f64};
use serde_json::Value;

pub(super) fn validate_drawing(value: &Value) -> Result<(), String> {
    let object = value
        .as_object()
        .ok_or("Worksheet drawing must be an object")?;
    ensure_keys(
        object,
        &["xEmu", "yEmu", "widthEmu", "heightEmu"],
        "Worksheet drawing",
    )?;
    for key in ["xEmu", "yEmu", "widthEmu", "heightEmu"] {
        let coordinate = required_f64(object, key, "Worksheet drawing coordinate")?;
        if coordinate.fract() != 0.0 || coordinate.abs() > 9_007_199_254_740_991.0 {
            return Err("Worksheet drawing coordinates must be safe EMU integers".into());
        }
        if matches!(key, "widthEmu" | "heightEmu") && coordinate <= 0.0 {
            return Err("Worksheet drawing dimensions must be positive".into());
        }
    }
    Ok(())
}
