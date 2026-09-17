use std::io::Cursor;

use image::{GenericImageView, ImageFormat};

use super::CreateVisualizationRequest;

pub fn validate_request(request: &CreateVisualizationRequest) -> Result<(), String> {
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
    let image = image::load_from_memory(bytes)
        .map_err(|error| format!("Generated image is invalid: {error}"))?;
    let (width, height) = image.dimensions();
    if width == 0 || height == 0 {
        return Err("Generated image dimensions are invalid".into());
    }
    let mut output = Cursor::new(Vec::new());
    image
        .write_to(&mut output, ImageFormat::Png)
        .map_err(|error| error.to_string())?;
    Ok((output.into_inner(), width, height))
}
