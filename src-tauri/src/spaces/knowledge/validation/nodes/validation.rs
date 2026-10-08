use serde_json::{Map, Value};

pub(super) fn reject_unknown<'a>(
    object: &Map<String, Value>,
    allowed: impl IntoIterator<Item = &'a str>,
    label: &str,
) -> Result<(), String> {
    let allowed = allowed.into_iter().collect::<Vec<_>>();
    if let Some(key) = object.keys().find(|key| !allowed.contains(&key.as_str())) {
        return Err(format!("{label} field '{key}' is not supported"));
    }
    Ok(())
}
