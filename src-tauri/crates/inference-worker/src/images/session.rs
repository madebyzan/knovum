use std::path::Path;

use ort::session::Session;

pub(super) fn load(
    path: &Path,
    threads: u8,
    inputs: &[&str],
    outputs: &[&str],
) -> Result<Session, String> {
    let session = crate::onnx::session(path, threads)?;
    let actual_inputs = session
        .inputs()
        .iter()
        .map(|value| value.name())
        .collect::<Vec<_>>();
    let actual_outputs = session
        .outputs()
        .iter()
        .map(|value| value.name())
        .collect::<Vec<_>>();
    if actual_inputs != inputs || !outputs.iter().all(|name| actual_outputs.contains(name)) {
        return Err("model_corrupt: ONNX graph interface differs from its specification".into());
    }
    Ok(session)
}

pub(super) fn floats(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes(chunk.try_into().expect("validated float bytes")))
        .collect()
}

pub(super) fn float_bytes(values: &[f32]) -> Result<Vec<u8>, String> {
    if values.iter().any(|value| !value.is_finite()) {
        return Err("invalid_output: non-finite tensor".into());
    }
    Ok(values
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect())
}
