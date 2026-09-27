mod content;
mod execution;
mod knowledge;
#[cfg(test)]
mod tests;

use super::{restore::contracts::RestorePackageRequest, ImportedPackageAsset};
use serde_json::{json, Value};
use std::collections::HashMap;

pub(super) struct IdMaps {
    source_space: String,
    target_space: String,
    entities: HashMap<&'static str, HashMap<String, String>>,
    assets: HashMap<String, String>,
}

const ENTITIES: &[&str] = &[
    "nodes",
    "structureNodes",
    "scenes",
    "relations",
    "derivations",
    "runs",
    "runSteps",
    "runAttempts",
    "externalAgentConversations",
    "externalAgentTurns",
    "assistantConversations",
    "assistantMessages",
    "objectTypes",
    "provenance",
    "chunks",
];
const COLLECTIONS: &[&str] = &[
    "nodes",
    "structureNodes",
    "layouts",
    "relationStyles",
    "scenes",
    "relations",
    "derivations",
    "runs",
    "runSteps",
    "runAttempts",
    "runEvents",
    "externalAgentConversations",
    "externalAgentTurns",
    "assistantConversations",
    "assistantMessages",
    "objectTypes",
    "provenance",
    "chunks",
    "knowledgeEvents",
];

pub(super) fn prepare(
    source: &str,
    assets: &[ImportedPackageAsset],
    target_space: &str,
) -> Result<RestorePackageRequest, String> {
    let mut value: Value = serde_json::from_str(source).map_err(|e| e.to_string())?;
    let root = value
        .as_object()
        .ok_or("Package manifest must be an object")?;
    if root.len() != COLLECTIONS.len() + 4
        || root.keys().any(|key| {
            !COLLECTIONS.contains(&key.as_str())
                && !["format", "version", "exportedAt", "space"].contains(&key.as_str())
        })
        || value["format"] != "knovum-space"
        || value["version"] != super::PACKAGE_VERSION
    {
        return Err("Unsupported Space package manifest".into());
    }
    validate_space(&value["space"])?;
    timestamp(&value, "exportedAt")?;
    let source_space = string(&value["space"], "id")?.to_owned();
    let mut maps = IdMaps {
        source_space,
        target_space: target_space.into(),
        entities: HashMap::new(),
        assets: assets
            .iter()
            .map(|a| (a.token.clone(), a.path.clone()))
            .collect(),
    };
    for &collection in ENTITIES {
        let mut ids = HashMap::new();
        for row in array(&value, collection)? {
            let id = string(row, "id")?;
            if ids
                .insert(id.into(), uuid::Uuid::new_v4().to_string())
                .is_some()
            {
                return Err(format!("Duplicate package {collection} identity"));
            }
        }
        maps.entities.insert(collection, ids);
    }
    knowledge::validate_source(&value)?;
    let original_types = array(&value, "objectTypes")?.clone();
    for &collection in COLLECTIONS {
        let rows = array_mut(&mut value, collection)?;
        for row in rows {
            let owner = row.get("spaceId");
            let owned = [
                "scenes",
                "relations",
                "derivations",
                "chunks",
                "provenance",
                "knowledgeEvents",
            ]
            .contains(&collection);
            if owned && owner.and_then(Value::as_str) != Some(maps.source_space.as_str()) {
                return Err("Package row belongs to another Space".into());
            }
            if let Some(owner) = owner {
                if !(collection == "objectTypes" && owner.is_null())
                    && owner.as_str() != Some(maps.source_space.as_str())
                {
                    return Err("Package row belongs to another Space".into());
                }
                object(row)?.remove("spaceId");
            }
            if collection == "objectTypes" {
                if row["system"] != false {
                    return Err("Package Object Type must be a custom type".into());
                }
                object(row)?.remove("system");
            }
            if ENTITIES.contains(&collection) {
                maps.field(row, "id", collection)?;
            }
        }
    }
    content::remap(&mut value, &mut maps, &original_types)?;
    execution::remap(&mut value, &maps)?;
    knowledge::remap(&mut value, &maps)?;
    let now = crate::clock::unix_time_millis()?;
    let space = &value["space"];
    let created = space["createdAt"]
        .as_i64()
        .filter(|v| *v >= 0)
        .ok_or("Invalid package Space timestamp")?;
    let title = string(space, "title")?.to_owned();
    value["space"] = json!({"id":target_space,"kind":"space","title":title,"lastOpenedAt":now,"createdAt":created,"updatedAt":now});
    let root = object(&mut value)?;
    root.remove("format");
    root.remove("version");
    root.remove("exportedAt");
    serde_json::from_value(value).map_err(|e| format!("Invalid Space restore contract: {e}"))
}

impl IdMaps {
    fn id(&self, collection: &str, source: &str) -> Result<String, String> {
        self.entities
            .get(collection)
            .and_then(|ids| ids.get(source))
            .cloned()
            .ok_or_else(|| format!("Package references unknown {collection} identity {source}"))
    }
    fn field(&self, row: &mut Value, field: &str, collection: &str) -> Result<(), String> {
        let source = string(row, field)?;
        row[field] = json!(self.id(collection, source)?);
        Ok(())
    }
    fn optional(&self, row: &mut Value, field: &str, collection: &str) -> Result<(), String> {
        if row.get(field).is_some_and(|v| !v.is_null()) {
            self.field(row, field, collection)?;
        }
        Ok(())
    }
    fn type_id(&self, source: &str) -> Result<String, String> {
        if source.starts_with("system:") {
            Ok(source.into())
        } else {
            self.id("objectTypes", source)
        }
    }
    fn asset(&self, source: &str) -> Result<String, String> {
        let token = source
            .strip_prefix(crate::spaces::assets::PACKAGE_ASSET_PREFIX)
            .ok_or("Managed package reference is not an asset token")?;
        self.assets
            .get(token)
            .cloned()
            .ok_or("Package references an unknown asset".into())
    }
}
fn string<'a>(value: &'a Value, field: &str) -> Result<&'a str, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("Package {field} must be a nonempty string"))
}
fn object(value: &mut Value) -> Result<&mut serde_json::Map<String, Value>, String> {
    value
        .as_object_mut()
        .ok_or("Package row must be an object".into())
}
fn array<'a>(value: &'a Value, field: &str) -> Result<&'a Vec<Value>, String> {
    value
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("Package {field} must be an array"))
}
fn array_mut<'a>(value: &'a mut Value, field: &str) -> Result<&'a mut Vec<Value>, String> {
    value
        .get_mut(field)
        .and_then(Value::as_array_mut)
        .ok_or_else(|| format!("Package {field} must be an array"))
}

fn timestamp(value: &Value, field: &str) -> Result<i64, String> {
    value
        .get(field)
        .and_then(Value::as_i64)
        .filter(|time| *time >= 0)
        .ok_or_else(|| format!("Invalid package {field} timestamp"))
}
fn validate_space(space: &Value) -> Result<(), String> {
    let fields = [
        "id",
        "kind",
        "title",
        "archivedAt",
        "lastOpenedAt",
        "createdAt",
        "updatedAt",
    ];
    let object = space.as_object().ok_or("Invalid package Space")?;
    if object.len() != fields.len() || object.keys().any(|key| !fields.contains(&key.as_str())) {
        return Err("Invalid package Space fields".into());
    }
    string(space, "id")?;
    if !["inbox", "space"].contains(&string(space, "kind")?)
        || string(space, "title")?.encode_utf16().count() > 512
    {
        return Err("Invalid package Space metadata".into());
    }
    for field in ["lastOpenedAt", "createdAt", "updatedAt"] {
        timestamp(space, field)?;
    }
    if !space["archivedAt"].is_null() {
        timestamp(space, "archivedAt")?;
    }
    Ok(())
}
