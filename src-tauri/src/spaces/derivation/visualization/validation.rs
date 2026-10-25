use super::GenerateVisualizationRequest;
use crate::spaces::derivation::source_inputs;

pub fn validate_request(request: &GenerateVisualizationRequest) -> Result<(), String> {
    if request.intent.trim().is_empty() {
        return Err("Visualization request is invalid".into());
    }
    source_inputs::require_led(&request.inputs)
}

pub fn canonical_png(bytes: &[u8]) -> Result<(Vec<u8>, u32, u32), String> {
    let image = super::super::image_operations::output::canonicalize(bytes)?;
    Ok((image.png, image.width, image.height))
}
