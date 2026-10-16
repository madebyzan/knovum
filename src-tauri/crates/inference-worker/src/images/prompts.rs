use knovum_inference_contracts::image_compute::{ImageSelectionPoint, ImageSelectionRect};

/// Builds the decoder prompt tensors. EfficientSAM takes prompt coordinates in source pixels and
/// distinguishes prompt kinds through labels: `1` marks a foreground point, `0` a background
/// point, and `2`/`3` the two corners of a box.
pub(super) fn build(
    points: &[ImageSelectionPoint],
    rect: Option<&ImageSelectionRect>,
    height: u32,
    width: u32,
) -> Result<(Vec<f32>, Vec<f32>), String> {
    let count = points.len() + usize::from(rect.is_some()) * 2;
    if !(1..=6).contains(&count) || rect.is_none() && !points.iter().any(|point| point.foreground) {
        return Err(
            "invalid_input: selection needs one to six prompts and a foreground point or box"
                .into(),
        );
    }
    let mut coords = Vec::with_capacity(count * 2);
    let mut labels = Vec::with_capacity(count);
    for point in points {
        coords.extend(pixel_position(point, height, width)?);
        labels.push(if point.foreground { 1.0 } else { 0.0 });
    }
    if let Some(rect) = rect {
        if [rect.x, rect.y, rect.width, rect.height]
            .iter()
            .any(|value| !value.is_finite())
            || rect.x < 0.0
            || rect.y < 0.0
            || rect.width <= 0.0
            || rect.height <= 0.0
            || rect.x + rect.width > 1.0
            || rect.y + rect.height > 1.0
        {
            return Err("invalid_input: selection box is outside image".into());
        }
        coords.extend([
            rect.x * width as f32,
            rect.y * height as f32,
            (rect.x + rect.width) * width as f32,
            (rect.y + rect.height) * height as f32,
        ]);
        labels.extend([2.0, 3.0]);
    }
    Ok((coords, labels))
}

/// Builds a single foreground point prompt, which is how layer discovery probes one candidate.
pub(super) fn foreground_point(
    point: &ImageSelectionPoint,
    height: u32,
    width: u32,
) -> Result<(Vec<f32>, Vec<f32>), String> {
    if !point.foreground {
        return Err("invalid_input: layer discovery seeds must be foreground points".into());
    }
    Ok((pixel_position(point, height, width)?.to_vec(), vec![1.0]))
}

fn pixel_position(
    point: &ImageSelectionPoint,
    height: u32,
    width: u32,
) -> Result<[f32; 2], String> {
    if !point.x.is_finite()
        || !point.y.is_finite()
        || !(0.0..=1.0).contains(&point.x)
        || !(0.0..=1.0).contains(&point.y)
    {
        return Err("invalid_input: selection point is outside image".into());
    }
    Ok([point.x * width as f32, point.y * height as f32])
}

#[cfg(test)]
#[path = "tests/prompts.rs"]
mod tests;
