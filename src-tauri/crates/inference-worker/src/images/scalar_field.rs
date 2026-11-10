//! A model's raw scalar field mapped to the full `[0, 1]` range. Depth and segmentation both
//! publish relative fields; a non-finite or constant field is an invalid output, never a fallback.

pub(super) fn normalize(values: &[f32]) -> Result<Vec<f32>, String> {
    if values.is_empty() || values.iter().any(|value| !value.is_finite()) {
        return Err("invalid_output: invalid scalar field".into());
    }
    let low = values.iter().copied().fold(f32::INFINITY, f32::min);
    let high = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    if high <= low || !(high - low).is_finite() {
        return Err("invalid_output: constant or overflowing scalar field".into());
    }
    Ok(values
        .iter()
        .map(|value| (value - low) / (high - low))
        .collect())
}
