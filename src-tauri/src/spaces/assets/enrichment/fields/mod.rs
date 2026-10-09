use serde_json::Value;

use super::{
    super::{kind::AssetKind, PreparedAsset},
    node_payload::set_optional,
};

pub(crate) fn apply_asset_fields(payload: &mut Value, asset: &PreparedAsset) -> Result<(), String> {
    let object = payload
        .as_object_mut()
        .ok_or_else(|| "Space node data must be an object".to_owned())?;
    set_optional(object, "previewPath", asset.preview_path.as_ref());
    set_optional(object, "previewText", asset.preview_text.as_ref());
    set_optional(object, "extractedText", asset.extracted_text.as_ref());
    if let Some(coverage) = &asset.document_coverage {
        object.insert("documentCoverage".into(), coverage.clone());
    } else {
        object.remove("documentCoverage");
    }
    if let Some(entries) = &asset.entries {
        object.insert(
            "entries".into(),
            serde_json::to_value(entries).map_err(|error| error.to_string())?,
        );
    }
    apply_intrinsic_geometry(
        object,
        asset.kind,
        asset.intrinsic_width,
        asset.intrinsic_height,
    )?;
    Ok(())
}

fn apply_intrinsic_geometry(
    object: &mut serde_json::Map<String, Value>,
    kind: AssetKind,
    width: Option<u32>,
    height: Option<u32>,
) -> Result<(), String> {
    if matches!(kind, AssetKind::Image | AssetKind::Video | AssetKind::File) {
        set_optional_u32(object, "intrinsicWidth", width);
        set_optional_u32(object, "intrinsicHeight", height);
        return Ok(());
    }
    if !matches!(kind, AssetKind::Audio | AssetKind::Code) {
        return Err(format!(
            "Asset kind {kind:?} cannot receive enrichment fields"
        ));
    }
    object.remove("intrinsicWidth");
    object.remove("intrinsicHeight");
    Ok(())
}

#[cfg(test)]
mod tests;

fn set_optional_u32(object: &mut serde_json::Map<String, Value>, key: &str, value: Option<u32>) {
    if let Some(value) = value {
        object.insert(key.into(), Value::from(value));
    } else {
        object.remove(key);
    }
}
