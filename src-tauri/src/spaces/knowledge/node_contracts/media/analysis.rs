use serde_json::Value;

use super::super::{
    scalars::{required_safe_nonnegative_integer, required_string},
    validation::reject_unknown,
};

pub(super) fn validate(value: Option<&Value>) -> Result<(), String> {
    let Some(value) = value else {
        return Ok(());
    };
    let object = value.as_object().ok_or("mediaAnalysis must be an object")?;
    reject_unknown(
        object,
        ["schemaVersion", "sourceRevision", "durationMs", "waveform"],
        "mediaAnalysis",
    )?;
    if object.get("schemaVersion").and_then(Value::as_u64) != Some(1) {
        return Err("mediaAnalysis schemaVersion must be 1".into());
    }
    required_string(object, "sourceRevision", 128)?;
    if required_safe_nonnegative_integer(object, "durationMs")? == 0 {
        return Err("mediaAnalysis durationMs must be positive".into());
    }
    let waveform = object
        .get("waveform")
        .and_then(Value::as_array)
        .ok_or("mediaAnalysis waveform must be an array")?;
    if waveform.is_empty() || waveform.len() > 1_200 {
        return Err("mediaAnalysis requires 1 to 1200 waveform samples".into());
    }
    if waveform.iter().any(|value| {
        value
            .as_f64()
            .is_none_or(|sample| !sample.is_finite() || !(0.0..=1.0).contains(&sample))
    }) {
        return Err("mediaAnalysis waveform samples must be finite values between 0 and 1".into());
    }
    Ok(())
}
