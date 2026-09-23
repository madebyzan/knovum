use serde_json::Value;

use crate::spaces::runs::{StepOutcome, WaitReason};

pub(super) fn completed(output: Value) -> Result<StepOutcome, String> {
    Ok(StepOutcome::Completed {
        output,
        selected_branch: None,
    })
}

pub(super) fn waiting(reason: WaitReason, checkpoint: Value) -> Result<StepOutcome, String> {
    Ok(StepOutcome::Waiting { reason, checkpoint })
}

pub(super) fn required_str<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("action-input-invalid: {key} is required"))
}
