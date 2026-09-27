use super::{array_mut, object, string, IdMaps};
use serde_json::{json, Value};

pub(super) fn remap(value: &mut Value, maps: &mut IdMaps, types: &[Value]) -> Result<(), String> {
    for row in array_mut(value, "nodes")? {
        let data = &mut row["data"];
        crate::spaces::knowledge::indexing::revision::validate_declared(data)?;
        restore_assets(data, maps)?;
        match data["kind"].as_str() {
            Some("object") => {
                let source = string(data, "objectTypeId")?.to_owned();
                data["objectTypeId"] = json!(maps.type_id(&source)?);
                if !source.starts_with("system:") {
                    let definition = types
                        .iter()
                        .find(|v| v["id"] == source)
                        .ok_or("Unknown object type")?;
                    for field in super::array(definition, "fields")? {
                        if field["kind"] == "node-ref" {
                            let key = string(field, "key")?;
                            if let Some(id) = data["properties"][key].as_str() {
                                data["properties"][key] = json!(maps.id("nodes", id)?);
                            }
                        }
                    }
                }
            }
            Some("portal") if data["targetSpaceId"].as_str() == Some(&maps.source_space) => {
                data["targetSpaceId"] = json!(maps.target_space);
                for (field, collection) in [
                    ("targetSceneId", "scenes"),
                    ("targetNodeId", "nodes"),
                    ("targetStructureNodeId", "structureNodes"),
                ] {
                    maps.optional(data, field, collection)?;
                }
            }
            Some("subtitle") => {
                maps.field(data, "sourceVideoNodeId", "nodes")?;
            }
            Some("credential") => {
                data["credentialConfigured"] = json!(false);
            }
            _ => {}
        }
        crate::spaces::knowledge::indexing::revision::apply(data)?;
    }
    for row in array_mut(value, "structureNodes")? {
        maps.optional(row, "parentId", "structureNodes")?;
        if row["kind"] == "content" {
            maps.field(row, "nodeId", "nodes")?;
        }
    }
    for row in array_mut(value, "layouts")? {
        maps.field(row, "structureNodeId", "structureNodes")?;
        if let Some(path) = row["appearance"]["background"].as_str() {
            row["appearance"]["background"] = json!(maps.asset(path)?);
        }
    }
    for row in array_mut(value, "relationStyles")? {
        maps.field(row, "relationId", "relations")?;
    }
    for row in array_mut(value, "scenes")? {
        object(row)?.remove("thumbnailPath");
        for id in row["focusStructureNodeIds"]
            .as_array_mut()
            .ok_or("Saved View focus must be an array")?
        {
            let source = id.as_str().ok_or("Saved View focus identity is invalid")?;
            let ids = maps
                .entities
                .get_mut("structureNodes")
                .ok_or("Missing structure identity map")?;
            // A detached navigation target stays detached, with a fresh identity in the imported Space.
            *id = json!(ids
                .entry(source.into())
                .or_insert_with(|| uuid::Uuid::new_v4().to_string()));
        }
        if let Some(ids) = row
            .get_mut("queryFilter")
            .and_then(|v| v.get_mut("objectTypeIds"))
            .and_then(Value::as_array_mut)
        {
            for id in ids {
                *id = json!(maps.type_id(id.as_str().ok_or("Invalid query type identity")?)?);
            }
        }
    }
    for row in array_mut(value, "relations")? {
        maps.field(row, "sourceId", "nodes")?;
        maps.field(row, "targetId", "nodes")?;
    }
    Ok(())
}

fn restore_assets(data: &mut Value, maps: &IdMaps) -> Result<(), String> {
    for field in ["assetPath", "faviconPath", "previewImagePath"] {
        if let Some(path) = data[field].as_str() {
            data[field] = json!(maps.asset(path)?);
        }
    }
    if data.get("origin").is_some_and(Value::is_object) {
        if let Some(path) = data["previewPath"].as_str() {
            data["previewPath"] = json!(maps.asset(path)?);
        }
    } else {
        object(data)?.remove("previewPath");
    }
    if let Some(content) = data.get_mut("tiptapContent") {
        restore_tiptap(content, maps)?;
    }
    if let Some(resources) = data
        .get_mut("documentCoverage")
        .and_then(|v| v.get_mut("resources"))
        .and_then(Value::as_array_mut)
    {
        for resource in resources {
            if let Some(path) = resource["path"].as_str() {
                resource["path"] = json!(maps.asset(path)?);
            }
        }
    }
    Ok(())
}
fn restore_tiptap(node: &mut Value, maps: &IdMaps) -> Result<(), String> {
    if node["type"] == "image" {
        if let Some(path) = node["attrs"]["assetPath"].as_str() {
            let path = maps.asset(path)?;
            node["attrs"]["assetPath"] = json!(path);
            node["attrs"]["src"] = json!(path);
        }
    }
    if let Some(children) = node.get_mut("content").and_then(Value::as_array_mut) {
        for child in children {
            restore_tiptap(child, maps)?;
        }
    }
    Ok(())
}
