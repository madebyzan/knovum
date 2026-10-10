use super::super::inputs::ResolvedInputs;
use serde_json::{Map, Value};

pub(super) fn request(
    inputs: &ResolvedInputs,
    request: &mut Map<String, Value>,
) -> Result<(), String> {
    let source = inputs
        .nodes
        .first()
        .ok_or("action-input-invalid: image is required")?;
    if inputs.nodes.len() != 1 || source.kind != "image" {
        return Err("action-input-invalid: image transform requires one image".into());
    }
    let revision = source
        .data
        .get("sourceRevision")
        .and_then(Value::as_str)
        .ok_or("Image asset revision is unavailable")?;
    *request = Map::from_iter([
        ("sourceNodeId".into(), source.id.clone().into()),
        ("sourceRevision".into(), revision.into()),
        (
            "sourceEdit".into(),
            source.data.get("imageEdit").cloned().unwrap_or(Value::Null),
        ),
    ]);
    Ok(())
}
