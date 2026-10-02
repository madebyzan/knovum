use std::collections::HashMap;

use serde_json::Value;
use sqlx::{Sqlite, Transaction};

use super::contracts::NodeContentChange;
use crate::{
    app_error::AppError,
    spaces::{
        db,
        structure::{
            self, Size, SizingMode, SpaceEditSnapshotResponse, StructureLayout, StructureNode,
        },
    },
};

const DOCUMENT_CAPTION_HEIGHT: f64 = 48.0;
const SIZE_EPSILON: f64 = 0.01;

pub(in crate::spaces) async fn sync_changed_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    changes: &[NodeContentChange],
) -> Result<Option<SpaceEditSnapshotResponse>, AppError> {
    let node_ids = changes
        .iter()
        .filter_map(|change| {
            let after = display_ratio(&change.data)?;
            (display_ratio(&change.expected_data) != Some(after)).then(|| change.id.clone())
        })
        .collect::<Vec<_>>();
    if node_ids.is_empty() {
        return Ok(None);
    }
    sync_node_ids_in_transaction(tx, space_id, &node_ids).await
}

pub(in crate::spaces) async fn sync_node_ids_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_ids: &[String],
) -> Result<Option<SpaceEditSnapshotResponse>, AppError> {
    let (mut snapshot, revision) = structure::repository::load_in_transaction(tx, space_id).await?;
    if !fit_in_snapshot(tx, space_id, &mut snapshot, node_ids).await? {
        return Ok(None);
    }
    let edit_revision =
        structure::repository::save_in_transaction(tx, space_id, revision, &snapshot).await?;
    Ok(Some(SpaceEditSnapshotResponse {
        edit_revision,
        structure: snapshot.structure,
        layouts: snapshot.layouts,
        relation_styles: snapshot.relation_styles,
    }))
}

pub(in crate::spaces) async fn fit_in_snapshot(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    snapshot: &mut structure::repository::SpaceStructureSnapshot,
    node_ids: &[String],
) -> Result<bool, AppError> {
    let mut payloads = HashMap::new();
    for node_id in node_ids {
        let encoded: Option<String> = sqlx::query_scalar(
            "SELECT data FROM spaces_nodes WHERE space_id=$1 AND id=$2 AND deleted_at IS NULL \
             AND kind IN ('image','video','file')",
        )
        .bind(space_id)
        .bind(node_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(db::sql_error)?;
        if let Some(encoded) = encoded {
            payloads.insert(
                node_id.clone(),
                serde_json::from_str(&encoded).map_err(db::json_error)?,
            );
        }
    }
    let structure_ids = snapshot
        .structure
        .iter()
        .filter_map(|node| match node {
            StructureNode::Content { id, node_id, .. } if payloads.contains_key(node_id) => {
                Some((id.as_str(), node_id.as_str()))
            }
            _ => None,
        })
        .collect::<HashMap<_, _>>();
    let mut changed = false;
    for layout in &mut snapshot.layouts {
        let StructureLayout::Content {
            structure_node_id,
            size,
            sizing_mode,
            ..
        } = layout
        else {
            continue;
        };
        let Some(node_id) = structure_ids.get(structure_node_id.as_str()) else {
            continue;
        };
        let data = &payloads[*node_id];
        let Some(next) = fitted_size(data, size, *sizing_mode) else {
            continue;
        };
        if same_size(size, &next) {
            continue;
        }
        *size = next;
        changed = true;
    }
    Ok(changed)
}

pub(in crate::spaces) fn fitted_size(
    data: &Value,
    current: &Size,
    sizing_mode: SizingMode,
) -> Option<Size> {
    let kind = data.get("kind")?.as_str()?;
    let ratio = display_ratio(data)?;
    match sizing_mode {
        SizingMode::Content => default_size(kind, ratio),
        SizingMode::Frame if kind == "file" => None,
        SizingMode::Frame => {
            let area = current.width * current.height;
            let width = (area * ratio).sqrt();
            Some(Size {
                width,
                height: width / ratio,
            })
        }
    }
}

fn default_size(kind: &str, ratio: f64) -> Option<Size> {
    let (preferred_long_edge, minimum_short_edge, max_long_edge, footer) = match kind {
        "image" => (280.0, 128.0, 480.0, 0.0),
        "video" => (304.0, 156.0, 560.0, 0.0),
        "file" => (272.0, 150.0, 480.0, DOCUMENT_CAPTION_HEIGHT),
        _ => return None,
    };
    let (mut width, mut height) = if ratio >= 1.0 {
        (preferred_long_edge, preferred_long_edge / ratio)
    } else {
        (preferred_long_edge * ratio, preferred_long_edge)
    };
    let short_edge = width.min(height);
    if short_edge < minimum_short_edge {
        let scale = minimum_short_edge / short_edge;
        width *= scale;
        height *= scale;
    }
    let long_edge = width.max(height);
    if long_edge > max_long_edge {
        let scale = max_long_edge / long_edge;
        width *= scale;
        height *= scale;
    }
    Some(Size {
        width: width.round(),
        height: (height + footer).round(),
    })
}

fn display_ratio(data: &Value) -> Option<f64> {
    let kind = data.get("kind")?.as_str()?;
    if !matches!(kind, "image" | "video" | "file") {
        return None;
    }
    let intrinsic_width = positive_number(data.get("intrinsicWidth")?)?;
    let intrinsic_height = positive_number(data.get("intrinsicHeight")?)?;
    let ratio = if kind == "image" {
        let edit = data.get("imageEdit").filter(|value| value.is_object());
        let crop = edit.and_then(|value| value.get("crop"));
        let width = intrinsic_width
            * crop
                .and_then(|value| value.get("width"))
                .and_then(positive_number)
                .unwrap_or(1.0);
        let height = intrinsic_height
            * crop
                .and_then(|value| value.get("height"))
                .and_then(positive_number)
                .unwrap_or(1.0);
        let radians = edit
            .and_then(|value| value.get("rotation"))
            .and_then(Value::as_f64)
            .unwrap_or(0.0)
            .to_radians();
        let cosine = radians.cos().abs();
        let sine = radians.sin().abs();
        let rotated_width = width * cosine + height * sine;
        let rotated_height = width * sine + height * cosine;
        rotated_width / rotated_height
    } else {
        intrinsic_width / intrinsic_height
    };
    (ratio.is_finite() && ratio > 0.0).then_some(ratio)
}

fn positive_number(value: &Value) -> Option<f64> {
    let number = value.as_f64()?;
    (number.is_finite() && number > 0.0).then_some(number)
}

fn same_size(left: &Size, right: &Size) -> bool {
    (left.width - right.width).abs() < SIZE_EPSILON
        && (left.height - right.height).abs() < SIZE_EPSILON
}

#[cfg(test)]
mod tests;
