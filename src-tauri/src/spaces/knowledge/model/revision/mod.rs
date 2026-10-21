use serde_json::{Map, Value};

const REVISION_PREFIX: &str = "node1:";
const LOCAL_TIPTAP_ATTRS: &[&str] = &["assetPath", "previewPath", "src"];

pub(crate) fn authoritative(node_payload: &Value) -> Result<Option<String>, String> {
    let Some(kind) = node_payload.get("kind").and_then(Value::as_str) else {
        return Err(
            "Space node data.kind is required before computing its Knowledge revision".into(),
        );
    };
    if matches!(kind, "credential" | "portal" | "text") {
        return Ok(None);
    }
    let object = node_payload
        .as_object()
        .ok_or_else(|| "Space node data must be an object".to_string())?;
    let mut projection = Map::new();
    projection.insert("kind".into(), Value::String(kind.to_owned()));
    insert_if_present(&mut projection, object, "title");
    insert_if_present(&mut projection, object, "tags");
    match kind {
        "note" => {
            if let Some(value) = object.get("tiptapContent") {
                projection.insert("tiptapContent".into(), sanitize(value));
            }
            insert_if_present(&mut projection, object, "plainText");
        }
        "markdown" => insert_if_present(&mut projection, object, "text"),
        "website" => {
            for key in [
                "url",
                "siteName",
                "description",
                "contentPreview",
                "websiteSections",
            ] {
                insert_if_present(&mut projection, object, key);
            }
        }
        "object" => {
            for key in ["objectTypeId", "summary", "status", "properties"] {
                insert_if_present(&mut projection, object, key);
            }
        }
        "image" | "video" | "audio" | "file" | "code" => {
            insert_if_present(&mut projection, object, "fileName");
            insert_if_present(&mut projection, object, "previewText");
            if matches!(kind, "video" | "audio") {
                for key in [
                    "origin",
                    "creator",
                    "durationSeconds",
                    "sourceRevision",
                    "mediaKnowledge",
                ] {
                    insert_if_present(&mut projection, object, key);
                }
            }
            if kind == "image" {
                insert_if_present(&mut projection, object, "sourceRevision");
                insert_if_present(&mut projection, object, "imageEdit");
            }
            if matches!(kind, "image" | "video") {
                insert_if_present(&mut projection, object, "visualKnowledge");
            }
        }
        _ => {}
    }
    let payload = stable_json(&Value::Object(projection));
    Ok(Some(format!(
        "{REVISION_PREFIX}{:016x}",
        fnv1a64(payload.as_bytes())
    )))
}

pub(crate) fn validate_declared(payload: &Value) -> Result<(), String> {
    let Some(expected) = authoritative(payload)? else {
        return Ok(());
    };
    let declared = payload
        .get("knowledgeRevision")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            "Semantic Space nodes require an authoritative knowledgeRevision".to_string()
        })?;
    if declared != expected {
        return Err(
            "Space node knowledgeRevision does not match its canonical semantic content".into(),
        );
    }
    Ok(())
}

pub(crate) fn apply(payload: &mut Value) -> Result<Option<String>, String> {
    let revision = authoritative(payload)?;
    let Some(object) = payload.as_object_mut() else {
        return Err("Space node data must be an object".into());
    };
    match &revision {
        Some(value) => {
            object.insert("knowledgeRevision".into(), Value::String(value.clone()));
        }
        None => {
            object.remove("knowledgeRevision");
        }
    }
    Ok(revision)
}

fn insert_if_present(target: &mut Map<String, Value>, source: &Map<String, Value>, key: &str) {
    if let Some(value) = source.get(key) {
        target.insert(key.to_owned(), value.clone());
    }
}

fn sanitize(value: &Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.iter().map(sanitize).collect()),
        Value::Object(object) => {
            let image_node = object.get("type").and_then(Value::as_str) == Some("image");
            let mut sanitized = Map::new();
            let mut keys = object.keys().collect::<Vec<_>>();
            keys.sort();
            for key in keys {
                let child = if key == "attrs" {
                    sanitize_attrs(&object[key], image_node)
                } else {
                    sanitize(&object[key])
                };
                sanitized.insert(key.clone(), child);
            }
            Value::Object(sanitized)
        }
        _ => value.clone(),
    }
}

fn sanitize_attrs(value: &Value, image_node: bool) -> Value {
    let Some(object) = value.as_object() else {
        return sanitize(value);
    };
    let managed_image = image_node
        && object
            .get("assetPath")
            .and_then(Value::as_str)
            .is_some_and(|value| !value.is_empty());
    let mut sanitized = Map::new();
    let mut keys = object.keys().collect::<Vec<_>>();
    keys.sort();
    for key in keys {
        if managed_image && LOCAL_TIPTAP_ATTRS.contains(&key.as_str()) {
            continue;
        }
        sanitized.insert(key.clone(), sanitize(&object[key]));
    }
    Value::Object(sanitized)
}

mod canonical;
use canonical::{fnv1a64, stable_json};

#[cfg(test)]
mod tests;
