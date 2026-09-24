use serde_json::Value;

pub(super) fn validate_unpublished_asset(output: &Value) -> Result<&str, String> {
    let output_path = output
        .get("assetPath")
        .and_then(Value::as_str)
        .filter(|path| !path.is_empty())
        .ok_or("Derived output path is unavailable")?;
    if output.get("processingState").and_then(Value::as_str) != Some("processing")
        || output.get("sourceRevision").is_some()
        || output.get("previewPath").is_some()
    {
        return Err("Derived output must be an unpublished asset".into());
    }
    Ok(output_path)
}

pub(super) fn validate_separate_asset(output: &Value, source_path: &str) -> Result<(), String> {
    if validate_unpublished_asset(output)? == source_path {
        return Err("Derived output must use a separate asset path".into());
    }
    Ok(())
}
