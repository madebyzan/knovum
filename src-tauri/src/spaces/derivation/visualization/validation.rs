use super::GenerateVisualizationRequest;

pub fn validate_request(request: &GenerateVisualizationRequest) -> Result<(), String> {
    let primary_count = request
        .inputs
        .iter()
        .filter(|input| input.role == "primary")
        .count();
    let primary_is_first = request
        .inputs
        .first()
        .is_some_and(|input| input.role == "primary");
    if request.intent.trim().is_empty()
        || request.inputs.is_empty()
        || primary_count != 1
        || !primary_is_first
    {
        return Err("Visualization request is invalid".into());
    }
    Ok(())
}

pub fn canonical_png(bytes: &[u8]) -> Result<(Vec<u8>, u32, u32), String> {
    let image = super::super::image_operations::output::canonicalize(bytes, None)?;
    Ok((image.png, image.width, image.height))
}
