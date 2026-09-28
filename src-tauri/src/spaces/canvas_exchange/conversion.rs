use super::acquisition::ImportResources;
use super::{
    contract::{JsonCanvasDocumentV1, JsonCanvasNodeV1, NodeFrameV1},
    file_import::ImportRequest,
    import::{ImportJsonCanvasNodesRequest, JsonCanvasNodeImport},
};
use crate::spaces::structure::repository::SpaceStructureSnapshot;
use serde_json::{json, Value};
use std::collections::HashMap;

pub(super) fn prepare(
    request: &ImportRequest,
    document: JsonCanvasDocumentV1,
    snapshot: SpaceStructureSnapshot,
    resources: &mut ImportResources<'_>,
) -> Result<ImportJsonCanvasNodesRequest, String> {
    let groups: Vec<_> = document
        .nodes
        .iter()
        .filter_map(|node| {
            if let JsonCanvasNodeV1::Group(group) = node {
                Some(group)
            } else {
                None
            }
        })
        .collect();
    let mut identities = HashMap::new();
    let parents = memberships(&document)?;
    for node in &document.nodes {
        identities.insert(node.frame().id.clone(), uuid::Uuid::new_v4().to_string());
    }
    let mut structure: Vec<Value> = serde_json::to_value(snapshot.structure)
        .map_err(|error| error.to_string())?
        .as_array()
        .ok_or("Invalid Space structure")?
        .clone();
    let mut layouts: Vec<Value> = serde_json::to_value(snapshot.layouts)
        .map_err(|error| error.to_string())?
        .as_array()
        .ok_or("Invalid Space layouts")?
        .clone();
    let mut root_order = structure
        .iter()
        .filter(|node| node["parentId"].is_null())
        .count();
    let mut child_orders: HashMap<String, usize> = HashMap::new();
    let mut nodes = Vec::new();
    for node in &document.nodes {
        super::import_control::check(resources.cancellation)?;
        let frame = node.frame();
        let id = identities
            .get(&frame.id)
            .ok_or("JSON Canvas identity is unavailable")?;
        let mut appearance = appearance(frame);
        if let JsonCanvasNodeV1::Group(group) = node {
            let title = group
                .label
                .as_deref()
                .map(str::trim)
                .filter(|title| !title.is_empty())
                .unwrap_or(&request.section_title);
            let background = if let Some(background) = &group.background {
                Some(
                    resources
                        .acquire(background)?
                        .path
                        .ok_or("JSON Canvas background has no managed path")?,
                )
            } else {
                None
            };
            appearance = section_appearance(frame, background, group.background_style.as_ref())?;
            structure.push(json!({"id":id,"kind":"section","parentId":null,"order":root_order,"locked":false,"title":title}));
            layouts.push(json!({"structureNodeId":id,"kind":"section","position":{"x":frame.x,"y":frame.y},"size":{"width":frame.width,"height":frame.height},"collapsed":false,"appearance":appearance}));
            root_order += 1;
            continue;
        }
        let (parent, x, y, order) = if let Some(parent_source) = parents.get(&frame.id) {
            let parent = identities
                .get(parent_source)
                .ok_or("JSON Canvas parent identity is unavailable")?;
            let group = groups
                .iter()
                .find(|group| group.frame.id == *parent_source)
                .ok_or("JSON Canvas parent group is unavailable")?;
            let order = child_orders.entry(parent.clone()).or_insert(0);
            let current = *order;
            *order += 1;
            (
                json!(parent),
                frame.x - group.frame.x,
                frame.y - group.frame.y,
                current,
            )
        } else {
            let order = root_order;
            root_order += 1;
            (Value::Null, frame.x, frame.y, order)
        };
        let data = node_payload(node, &request.note_title, resources)?;
        let prepared = json!({"id":id,"kind":data["kind"],"initialX":frame.x,"initialY":frame.y,"width":frame.width,"height":frame.height,"sizingMode":"frame","appearance":appearance,"fileSubpath":node.subpath(),"data":data,"sourceChunks":[]});
        nodes.push(JsonCanvasNodeImport {
            source_id: frame.id.clone(),
            node: serde_json::from_value(prepared).map_err(|error| error.to_string())?,
        });
        structure.push(json!({"id":id,"kind":"content","parentId":parent,"order":order,"locked":false,"nodeId":id}));
        let mut layout = json!({"structureNodeId":id,"kind":"content","transform":{"x":x,"y":y,"rotation":0,"scale":1},"size":{"width":frame.width,"height":frame.height},"sizingMode":"frame","appearance":appearance});
        if let Some(subpath) = node.subpath() {
            layout["fileSubpath"] = json!(subpath);
        }
        layouts.push(layout);
    }
    let relations = document
        .edges
        .iter()
        .map(|edge| relation(edge, &identities, &request.space_id))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ImportJsonCanvasNodesRequest {
        space_id: request.space_id.clone(),
        mutation_id: request.mutation_id.clone(),
        base_revision: request.base_revision,
        structure_nodes: serde_json::from_value(json!(structure))
            .map_err(|error| error.to_string())?,
        layouts: serde_json::from_value(json!(layouts)).map_err(|error| error.to_string())?,
        relation_styles: snapshot.relation_styles,
        nodes,
        relations,
    })
}

fn contains(parent: &NodeFrameV1, child: &NodeFrameV1) -> bool {
    child.x >= parent.x
        && child.y >= parent.y
        && child.x + child.width <= parent.x + parent.width
        && child.y + child.height <= parent.y + parent.height
}
fn appearance(frame: &NodeFrameV1) -> Value {
    match frame.color.as_deref() {
        None => json!({}),
        Some("1" | "2") => json!({"fill":"pink"}),
        Some("3") => json!({"fill":"yellow"}),
        Some("4") => json!({"fill":"green"}),
        Some("5") => json!({"fill":"blue"}),
        Some("6") => json!({"fill":"gray"}),
        Some(color) => json!({"customFill":color}),
    }
}
fn section_appearance(
    frame: &NodeFrameV1,
    background: Option<String>,
    style: Option<&super::contract::BackgroundStyleV1>,
) -> Result<Value, String> {
    let mut section_appearance = appearance(frame);
    if let Some(background) = background {
        section_appearance["background"] = json!(background);
    }
    if let Some(style) = style {
        section_appearance["backgroundStyle"] =
            serde_json::to_value(style).map_err(|error| error.to_string())?;
    }
    Ok(section_appearance)
}

fn node_payload(
    node: &JsonCanvasNodeV1,
    note_title: &str,
    resources: &mut ImportResources<'_>,
) -> Result<Value, String> {
    let asset = if let JsonCanvasNodeV1::File(node) = node {
        Some(resources.acquire(&node.file)?)
    } else {
        None
    };
    super::content::payload(node, note_title, asset)
}

fn memberships(document: &JsonCanvasDocumentV1) -> Result<HashMap<String, String>, String> {
    let groups: Vec<_> = document
        .nodes
        .iter()
        .filter_map(|node| {
            if let JsonCanvasNodeV1::Group(group) = node {
                Some(group)
            } else {
                None
            }
        })
        .collect();
    let mut parents = HashMap::new();
    for node in &document.nodes {
        let frame = node.frame();
        if let JsonCanvasNodeV1::Group(_) = node {
            if frame.width < 240.0 || frame.height < 160.0 {
                return Err("JSON Canvas group is too small to become a Section".into());
            }
        } else {
            let containing: Vec<_> = groups
                .iter()
                .filter(|group| contains(&group.frame, frame))
                .collect();
            if containing.len() > 1 {
                return Err("JSON Canvas content has ambiguous visual group membership".into());
            }
            if let Some(group) = containing.first() {
                parents.insert(frame.id.clone(), group.frame.id.clone());
            }
        }
    }
    for edge in &document.edges {
        if groups
            .iter()
            .any(|group| group.frame.id == edge.from_node || group.frame.id == edge.to_node)
        {
            return Err("JSON Canvas edges cannot target visual groups".into());
        }
    }
    Ok(parents)
}

fn relation(
    edge: &super::contract::JsonCanvasEdgeV1,
    identities: &HashMap<String, String>,
    space: &str,
) -> Result<crate::spaces::commands::relations::SpaceRelationInput, String> {
    Ok(crate::spaces::commands::relations::SpaceRelationInput {
        id: uuid::Uuid::new_v4().to_string(),
        space_id: space.into(),
        source_id: identities
            .get(&edge.from_node)
            .ok_or("Unknown JSON Canvas edge source")?
            .clone(),
        target_id: identities
            .get(&edge.to_node)
            .ok_or("Unknown JSON Canvas edge target")?
            .clone(),
        kind: "related".into(),
        rationale: edge.label.clone().unwrap_or_default(),
        status: "active".into(),
        confidence: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_group_background_styles_with_the_managed_resource() {
        let frame: NodeFrameV1 =
            serde_json::from_value(json!({"id":"section","x":0,"y":0,"width":400,"height":300}))
                .unwrap();
        for style in ["cover", "ratio", "repeat"] {
            let parsed = serde_json::from_value(json!(style)).unwrap();
            let appearance = section_appearance(
                &frame,
                Some("/managed/background.png".into()),
                Some(&parsed),
            )
            .unwrap();
            assert_eq!(appearance["background"], "/managed/background.png");
            assert_eq!(appearance["backgroundStyle"], style);
        }
    }
    #[test]
    fn adopts_unambiguous_section_membership_and_rejects_group_edges() {
        let mut document: JsonCanvasDocumentV1 = serde_json::from_value(json!({"nodes":[{"id":"section","type":"group","x":0,"y":0,"width":400,"height":300},{"id":"note","type":"text","text":"Evidence","x":20,"y":40,"width":200,"height":120}]})).unwrap();
        assert_eq!(
            memberships(&document).unwrap().get("note").unwrap(),
            "section"
        );
        document.edges.push(
            serde_json::from_value(json!({"id":"edge","fromNode":"section","toNode":"note"}))
                .unwrap(),
        );
        assert!(memberships(&document)
            .unwrap_err()
            .contains("visual groups"));
    }
    #[test]
    fn maps_edges_to_semantic_relations_and_rejects_unknown_targets() {
        let edge = serde_json::from_value(
            json!({"id":"edge","fromNode":"a","toNode":"b","label":"supports"}),
        )
        .unwrap();
        let identities =
            HashMap::from([("a".into(), "node-a".into()), ("b".into(), "node-b".into())]);
        let mapped = relation(&edge, &identities, "space").unwrap();
        assert_eq!(mapped.source_id, "node-a");
        assert_eq!(mapped.target_id, "node-b");
        assert_eq!(mapped.kind, "related");
        assert_eq!(mapped.rationale, "supports");
        assert!(relation(&edge, &HashMap::new(), "space").is_err());
    }
    #[test]
    fn rejects_ambiguous_membership_and_undersized_sections() {
        let document = serde_json::from_value(json!({"nodes":[{"id":"left","type":"group","x":0,"y":0,"width":400,"height":300},{"id":"right","type":"group","x":0,"y":0,"width":400,"height":300},{"id":"note","type":"text","text":"Evidence","x":20,"y":40,"width":200,"height":120}]})).unwrap();
        assert!(memberships(&document).unwrap_err().contains("ambiguous"));
        let document = serde_json::from_value(
            json!({"nodes":[{"id":"small","type":"group","x":0,"y":0,"width":239,"height":160}]}),
        )
        .unwrap();
        assert!(memberships(&document).unwrap_err().contains("too small"));
    }
}
