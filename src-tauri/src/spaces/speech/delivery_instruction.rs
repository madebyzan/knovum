/// Whether a line spoken by this voice can carry a delivery instruction: the engine must take a
/// style, and VoxCPM2 takes one only when it clones from the recording alone — with the exact
/// transcript it continues the reference as spoken.
pub(in crate::spaces) fn supports(engine: &str, source: &str, transcript: Option<&str>) -> bool {
    use crate::ai::contracts::speech_wire::SpeechSynthesisEngine as Engine;
    Engine::from_id(engine).is_some_and(|engine| {
        source == "reference"
            && engine.accepts_style()
            && (engine != Engine::VoxCpm2 || transcript.is_none())
    })
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
