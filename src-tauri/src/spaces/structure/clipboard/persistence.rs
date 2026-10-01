use std::collections::HashMap;

use serde_json::Value;
use sqlx::{Sqlite, Transaction};

use super::{ClipboardNode, ClipboardRelation};
use crate::spaces::structure::{
    planner::spatial::world_transform, planner::tree_policy::Index, StructureLayout, StructureNode,
};
use crate::spaces::{commands::relations::SpaceRelationInput, db};

pub(super) async fn insert_nodes(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    nodes: &[ClipboardNode],
    structure: &[StructureNode],
    layouts: &[StructureLayout],
    now: i64,
) -> Result<Vec<Value>, String> {
    let index = Index::new(structure, layouts);
    let by_node: HashMap<&str, &str> = structure
        .iter()
        .filter_map(|structure_node| match structure_node {
            StructureNode::Content { id, node_id, .. } => Some((node_id.as_str(), id.as_str())),
            _ => None,
        })
        .collect();
    let mut response = Vec::new();
    for node in nodes {
        let kind = node
            .data
            .get("kind")
            .and_then(Value::as_str)
            .ok_or("Clipboard node kind is missing")?
            .to_owned();
        let object_type_id = (kind == "object")
            .then(|| {
                node.data
                    .get("objectTypeId")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            })
            .flatten();
        let world = world_transform(structure, layouts, &index, by_node[node.id.as_str()])?;
        let raw = serde_json::to_string(&node.data).map_err(db::json_error)?;
        sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,object_type_id,initial_x,initial_y,data,created_at,updated_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$8)").bind(&node.id).bind(space_id).bind(&kind).bind(object_type_id).bind(world.x).bind(world.y).bind(raw).bind(now).execute(&mut **tx).await.map_err(db::sql_error)?;
        response.push(serde_json::json!({"id":node.id,"type":kind,"data":node.data,"initialPosition":{"x":world.x,"y":world.y}}));
    }
    for node in nodes {
        let kind = node
            .data
            .get("kind")
            .and_then(Value::as_str)
            .ok_or("Clipboard node kind is missing")?;
        crate::spaces::knowledge::contracts::validate_node(tx, space_id, kind, &node.data).await?;
    }
    Ok(response)
}

pub(super) async fn insert_relations(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    relations: &[ClipboardRelation],
    now: i64,
) -> Result<Vec<Value>, String> {
    let mut response = Vec::new();
    for relation in relations {
        let input = SpaceRelationInput {
            id: relation.id.clone(),
            space_id: space_id.to_owned(),
            source_id: relation.source_id.clone(),
            target_id: relation.target_id.clone(),
            kind: crate::spaces::knowledge::contracts::RelationKind::parse(&relation.kind)?,
            rationale: relation.rationale.clone(),
            status: crate::spaces::knowledge::contracts::RelationStatus::parse(&relation.status)?,
            confidence: relation.confidence,
        };
        crate::spaces::commands::relations::create_relation_in_transaction(
            tx, &input, "human", None, now,
        )
        .await?;
        let (source, target) = crate::spaces::commands::relations::canonical_endpoints(&input);
        response.push(serde_json::json!({"id":input.id,"spaceId":space_id,"sourceId":source,"targetId":target,"kind":input.kind,"rationale":input.rationale,"status":input.status,"statusBeforeInvalidation":null,"confidence":input.confidence,"actorKind":"human","actorId":null,"createdAt":now,"updatedAt":now}));
    }
    Ok(response)
}
