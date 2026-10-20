use std::collections::HashMap;

use serde_json::{json, Value};

use super::acquisition::ImportResources;
use super::{
    contracts::{JsonCanvasDocumentV1, JsonCanvasNodeV1, NodeFrameV1},
    file_import::ImportRequest,
    import::{ImportJsonCanvasNodesRequest, JsonCanvasNodeImport},
};
use crate::foundation::app_error::AppError;
use crate::spaces::{
    nodes::contracts::InsertNode,
    structure::{
        repository::SpaceStructureSnapshot, LocalTransform, Point, Size, SizingMode,
        StructureLayout, StructureNode,
    },
};

pub(super) fn prepare(
    request: &ImportRequest,
    document: JsonCanvasDocumentV1,
    snapshot: SpaceStructureSnapshot,
    resources: &mut ImportResources<'_>,
) -> Result<ImportJsonCanvasNodesRequest, AppError> {
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
    let mut structure = snapshot.structure;
    let mut layouts = snapshot.layouts;
    let mut root_order = structure
        .iter()
        .filter(|node| node.parent_id().is_none())
        .count();
    let mut next_order_by_parent: HashMap<String, usize> = HashMap::new();
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
            structure.push(StructureNode::Section {
                id: id.clone(),
                parent_id: None,
                order: u32::try_from(root_order).map_err(|_| "JSON Canvas order exceeds limit")?,
                locked: false,
                title: title.to_owned(),
            });
            layouts.push(StructureLayout::Section {
                structure_node_id: id.clone(),
                position: Point {
                    x: frame.x,
                    y: frame.y,
                },
                size: Size {
                    width: frame.width,
                    height: frame.height,
                },
                collapsed: false,
                appearance: Some(appearance),
            });
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
            let order = next_order_by_parent.entry(parent.clone()).or_insert(0);
            let current = *order;
            *order += 1;
            (
                Some(parent.clone()),
                frame.x - group.frame.x,
                frame.y - group.frame.y,
                current,
            )
        } else {
            let order = root_order;
            root_order += 1;
            (None, frame.x, frame.y, order)
        };
        let data = node_payload(node, &request.note_title, resources)?;
        let prepared = InsertNode {
            id: id.clone(),
            kind: data["kind"]
                .as_str()
                .ok_or("JSON Canvas node kind is invalid")?
                .to_owned(),
            initial_x: frame.x,
            initial_y: frame.y,
            width: frame.width,
            height: frame.height,
            sizing_mode: SizingMode::Frame,
            appearance: Some(appearance.clone()),
            file_subpath: node.subpath().map(str::to_owned),
            data,
            source_chunks: Vec::new(),
        };
        nodes.push(JsonCanvasNodeImport {
            source_id: frame.id.clone(),
            node: prepared,
        });
        structure.push(StructureNode::Content {
            id: id.clone(),
            parent_id: parent,
            order: u32::try_from(order).map_err(|_| "JSON Canvas order exceeds limit")?,
            locked: false,
            node_id: id.clone(),
        });
        layouts.push(StructureLayout::Content {
            structure_node_id: id.clone(),
            transform: LocalTransform {
                x,
                y,
                rotation: 0.0,
                scale: 1.0,
            },
            size: Size {
                width: frame.width,
                height: frame.height,
            },
            sizing_mode: SizingMode::Frame,
            appearance: Some(appearance),
            file_subpath: node.subpath().map(str::to_owned),
        });
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
        structure_nodes: structure,
        layouts,
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
    style: Option<&super::contracts::BackgroundStyleV1>,
) -> Result<Value, AppError> {
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
) -> Result<Value, AppError> {
    let asset = if let JsonCanvasNodeV1::File(node) = node {
        Some(resources.acquire(&node.file)?)
    } else {
        None
    };
    Ok(super::content::payload(node, note_title, asset)?)
}

fn memberships(document: &JsonCanvasDocumentV1) -> Result<HashMap<String, String>, AppError> {
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
    edge: &super::contracts::JsonCanvasEdgeV1,
    node_id_by_source_id: &HashMap<String, String>,
    space: &str,
) -> Result<crate::spaces::commands::relations::SpaceRelationInput, AppError> {
    Ok(crate::spaces::commands::relations::SpaceRelationInput {
        id: uuid::Uuid::new_v4().to_string(),
        space_id: space.into(),
        source_id: node_id_by_source_id
            .get(&edge.from_node)
            .ok_or("Unknown JSON Canvas edge source")?
            .clone(),
        target_id: node_id_by_source_id
            .get(&edge.to_node)
            .ok_or("Unknown JSON Canvas edge target")?
            .clone(),
        kind: crate::spaces::knowledge::contracts::RelationKind::Related,
        rationale: edge.label.clone().unwrap_or_default(),
        status: crate::spaces::knowledge::contracts::RelationStatus::Active,
        confidence: None,
    })
}

#[cfg(test)]
mod tests;
