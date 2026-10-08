pub(in crate::spaces) fn supports(engine: &str, source: &str, transcript: Option<&str>) -> bool {
    engine == "voxcpm2" && source == "reference" && transcript.is_none()
}

pub(in crate::spaces) fn validate(
    instruction: Option<&str>,
    supported: bool,
) -> Result<Option<&str>, String> {
    let Some(instruction) = instruction else {
        return Ok(None);
    };
    let instruction = instruction.trim();
    if !(1..=500).contains(&instruction.chars().count()) {
        return Err("Delivery instruction must contain between 1 and 500 characters".into());
    }
    if !supported {
        return Err("This voice revision does not support delivery instructions".into());
    }
    Ok(Some(instruction))
}
