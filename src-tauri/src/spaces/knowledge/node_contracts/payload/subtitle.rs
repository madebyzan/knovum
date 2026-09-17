use serde_json::{Map, Value};

use super::super::scalars::{required_number_range, required_string};
use super::super::validation::reject_unknown;

pub(super) fn validate(object: &Map<String, Value>) -> Result<(), String> {
    required_string(object, "sourceVideoNodeId", 128)?;
    required_string(object, "sourceVideoRevision", 128)?;
    required_string(object, "language", 64)?;
    required_number_range(object, "durationMs", 1.0, i64::MAX as f64)?;
    let duration = integer(object, "durationMs")?;
    let cues = object
        .get("cues")
        .and_then(Value::as_array)
        .ok_or("Subtitle cues are required")?;
    if cues.is_empty() || cues.len() > 100_000 {
        return Err("Subtitle cue count is invalid".into());
    }
    let mut previous = 0;
    for cue in cues {
        let cue = cue.as_object().ok_or("Subtitle cue must be an object")?;
        reject_unknown(cue, ["startMs", "endMs", "text"], "Subtitle cue")?;
        required_number_range(cue, "startMs", 0.0, duration as f64)?;
        required_number_range(cue, "endMs", 1.0, duration as f64)?;
        let start = integer(cue, "startMs")?;
        let end = integer(cue, "endMs")?;
        required_string(cue, "text", 16_000)?;
        if end <= start || start < previous {
            return Err("Subtitle cues must be ordered and bounded".into());
        }
        previous = start;
    }
    Ok(())
}

fn integer(object: &Map<String, Value>, key: &str) -> Result<u64, String> {
    object
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("Subtitle {key} must be an integer"))
}
