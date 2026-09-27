use super::{array, array_mut, string, IdMaps};
use serde_json::{json, Value};
use std::collections::HashMap;

pub(super) fn remap(value: &mut Value, maps: &IdMaps) -> Result<(), String> {
    let revisions = array(value, "nodes")?
        .iter()
        .filter_map(|v| {
            Some((
                v["id"].as_str()?.to_owned(),
                v["data"]["knowledgeRevision"].as_str()?.to_owned(),
            ))
        })
        .collect::<HashMap<_, _>>();
    for row in array_mut(value, "chunks")? {
        maps.field(row, "nodeId", "nodes")?;
        remap_revision(row, "nodeId", &revisions)?;
    }
    for row in array_mut(value, "provenance")? {
        let collection = entity_collection(string(row, "targetKind")?)?;
        maps.field(row, "targetId", collection)?;
        maps.field(row, "sourceNodeId", "nodes")?;
        remap_revision(row, "sourceNodeId", &revisions)?;
    }
    for row in array_mut(value, "knowledgeEvents")? {
        let kind = string(row, "entityKind")?;
        let collection = entity_collection(kind)?;
        if kind == "type" {
            row["entityId"] = json!(maps.type_id(string(row, "entityId")?)?);
        } else {
            maps.field(row, "entityId", collection)?;
        }
        row["id"] = json!(uuid::Uuid::new_v4().to_string());
        if row["entityKind"] == "provenance" && row["eventKind"] == "created" {
            let payload = row.get_mut("payload").ok_or("Event payload is missing")?;
            let target = entity_collection(string(payload, "targetKind")?)?;
            maps.field(payload, "targetId", target)?;
        }
    }
    for row in array_mut(value, "assistantMessages")? {
        if let Some(citations) = row.get_mut("content").and_then(|v| v.get_mut("citations")) {
            remap_citations(citations, maps, &revisions)?;
        }
    }
    for row in array_mut(value, "derivations")? {
        if let Some(citations) = row
            .get_mut("groundedDocument")
            .and_then(|v| v.get_mut("citations"))
        {
            remap_citations(citations, maps, &revisions)?;
        }
    }
    Ok(())
}
fn entity_collection(kind: &str) -> Result<&'static str, String> {
    match kind {
        "node" => Ok("nodes"),
        "relation" => Ok("relations"),
        "provenance" => Ok("provenance"),
        "type" => Ok("objectTypes"),
        _ => Err("Invalid package entity kind".into()),
    }
}
fn remap_revision(
    row: &mut Value,
    node_field: &str,
    revisions: &HashMap<String, String>,
) -> Result<(), String> {
    if row["sourceRevision"]
        .as_str()
        .is_some_and(|v| v.starts_with("node1:"))
    {
        let revision = revisions
            .get(string(row, node_field)?)
            .ok_or("Canonical reference node has no Knowledge revision")?;
        row["sourceRevision"] = json!(revision);
        if let Some(locator) = row.get_mut("locator").filter(|v| v.is_object()) {
            locator["sourceRevision"] = json!(revision);
        }
    }
    Ok(())
}
fn remap_citations(
    value: &mut Value,
    maps: &IdMaps,
    revisions: &HashMap<String, String>,
) -> Result<(), String> {
    for citation in value.as_array_mut().ok_or("Citations must be an array")? {
        if citation["state"] == "detached" {
            continue;
        }
        let source = string(citation, "nodeId")?.to_owned();
        let canonical = string(citation, "chunkId")? == format!("node:{source}");
        maps.field(citation, "nodeId", "nodes")?;
        if canonical {
            let node = string(citation, "nodeId")?.to_owned();
            citation["chunkId"] = json!(format!("node:{node}"));
            let revision = revisions
                .get(&node)
                .ok_or("Canonical citation node has no Knowledge revision")?;
            citation["sourceRevision"] = json!(revision);
            citation["locator"]["sourceRevision"] = json!(revision);
        } else {
            maps.field(citation, "chunkId", "chunks")?;
            remap_revision(citation, "nodeId", revisions)?;
        }
    }
    Ok(())
}

pub(super) fn validate_source(value: &Value) -> Result<(), String> {
    let nodes = array(value, "nodes")?
        .iter()
        .map(|node| Ok((string(node, "id")?, &node["data"]["knowledgeRevision"])))
        .collect::<Result<HashMap<_, _>, String>>()?;
    for message in array(value, "assistantMessages")? {
        if let Some(citations) = message
            .get("content")
            .and_then(|content| content.get("citations"))
        {
            crate::spaces::knowledge::grounding::validate_citation_snapshots(citations)?;
            validate_citations(citations, &nodes)?;
        }
    }
    for derivation in array(value, "derivations")? {
        if let Some(citations) = derivation
            .get("groundedDocument")
            .and_then(|document| document.get("citations"))
        {
            crate::spaces::knowledge::grounding::validate_citation_snapshots(citations)?;
            validate_citations(citations, &nodes)?;
        }
    }
    Ok(())
}
fn validate_citations(citations: &Value, nodes: &HashMap<&str, &Value>) -> Result<(), String> {
    for citation in citations.as_array().ok_or("Citations must be an array")? {
        if citation["state"] == "detached" {
            continue;
        }
        let node = string(citation, "nodeId")?;
        let revision = nodes
            .get(node)
            .ok_or("Citation references an unavailable node")?;
        if string(citation, "chunkId")? == format!("node:{node}")
            && (citation["sourceRevision"] != **revision
                || citation["locator"]["sourceRevision"] != **revision)
        {
            return Err("Package canonical citation revision is stale".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rebinds_canonical_citations_and_preserves_detached_snapshots() {
        let maps = IdMaps {
            source_space: "old".into(),
            target_space: "new".into(),
            assets: HashMap::new(),
            entities: HashMap::from([(
                "nodes",
                HashMap::from([("old-node".into(), "new-node".into())]),
            )]),
        };
        let revisions = HashMap::from([("new-node".into(), "node1:new".into())]);
        let detached = json!({"index":2,"state":"detached","nodeTitle":"Historical","sourceRevision":"node1:old","quote":"Snapshot"});
        let mut citations = json!([{"index":1,"state":"current","nodeId":"old-node","chunkId":"node:old-node","sourceRevision":"node1:old","locator":{"sourceRevision":"node1:old"}},detached]);
        remap_citations(&mut citations, &maps, &revisions).unwrap();
        assert_eq!(citations[0]["nodeId"], "new-node");
        assert_eq!(citations[0]["chunkId"], "node:new-node");
        assert_eq!(citations[0]["sourceRevision"], "node1:new");
        assert_eq!(citations[0]["locator"]["sourceRevision"], "node1:new");
        assert_eq!(citations[1], detached);
    }
    #[test]
    fn rejects_stale_canonical_snapshot_before_rebinding() {
        let revision = json!("node1:current");
        let nodes = HashMap::from([("node", &revision)]);
        let citations = json!([{"state":"current","nodeId":"node","chunkId":"node:node","sourceRevision":"node1:stale","locator":{"sourceRevision":"node1:stale"}}]);
        assert!(validate_citations(&citations, &nodes)
            .unwrap_err()
            .contains("stale"));
    }
    #[test]
    fn rejects_malformed_grounded_document_citations_before_remapping() {
        let value = json!({"nodes":[{"id":"node","data":{"knowledgeRevision":"node1:current"}}],"assistantMessages":[],"derivations":[{"groundedDocument":{"citations":[{"state":"current","nodeId":"node","chunkId":"node:node","nodeTitle":"Source","sourceRevision":"node1:current","quote":"Evidence","locator":"invalid"}]}}]});
        assert!(validate_source(&value)
            .unwrap_err()
            .contains("locator must be an object"));
    }
}
