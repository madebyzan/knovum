use super::{array_mut, object, string, IdMaps};
use serde_json::{json, Value};

pub(super) fn remap(value: &mut Value, maps: &mut IdMaps, types: &[Value]) -> Result<(), String> {
    for row in array_mut(value, "nodes")? {
        let data = &mut row["data"];
        // Markdown asset destinations are rewritten during export and import. The imported
        // text receives a new canonical revision below; the packaged revision still anchors
        // the source citations until knowledge::remap rebases them.
        let packaged_markdown = data["kind"] == "markdown"
            && data["text"]
                .as_str()
                .is_some_and(|text| text.contains(crate::spaces::assets::PACKAGE_ASSET_PREFIX));
        if !packaged_markdown {
            crate::spaces::knowledge::model::revision::validate_declared(data)?;
        } else if data["knowledgeRevision"].as_str().is_none_or(|revision| {
            revision.strip_prefix("node1:").is_none_or(|digest| {
                digest.len() != 16 || !digest.bytes().all(|b| b.is_ascii_hexdigit())
            })
        }) {
            return Err("Package Markdown node requires a Knowledge revision".into());
        }
        crate::spaces::assets::rewrite_node_asset_paths(data, |path| maps.asset(path))?;
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
            Some("credential") => {
                data["credentialConfigured"] = json!(false);
            }
            _ => {}
        }
        crate::spaces::knowledge::model::revision::apply(data)?;
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
