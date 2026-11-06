use std::collections::HashMap;

use serde_json::Value;
use sqlx::{Sqlite, Transaction};

use super::contracts::NodeContentChange;
use crate::{
    foundation::app_error::AppError,
    spaces::{
        db,
        structure::{
            self, Size, SizingMode, SpaceEditSnapshotResponse, StructureLayout, StructureNode,
        },
    },
};

const SIZE_EPSILON: f64 = 0.01;

/// How the canvas sizes content that shows a picture of itself, when nobody has sized it: the long
/// edge takes its preferred length, a very elongated picture is widened until its short edge is
/// legible, and never past the longest edge a default may take.
#[derive(Clone, Copy, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Footprint {
    pub preferred_long_edge: f64,
    pub minimum_short_edge: f64,
    pub maximum_long_edge: f64,
    /// Chrome below the picture, outside its aspect.
    pub caption: f64,
    /// A picture made of pixels is never shown larger than its own pixels: enlarging only blurs
    /// it. A rendered page has no such limit — its size is page geometry, not pixels.
    pub pixel_bound: bool,
}

/// The footprint of each kind of pictured content, keyed by node kind. The renderer sizes a node it
/// is about to create by the same table, exported with the contracts.
pub(crate) const FOOTPRINTS: [(&str, Footprint); 3] = [
    (
        "image",
        Footprint {
            preferred_long_edge: 280.0,
            minimum_short_edge: 128.0,
            maximum_long_edge: 480.0,
            caption: 0.0,
            pixel_bound: true,
        },
    ),
    (
        "video",
        Footprint {
            preferred_long_edge: 304.0,
            minimum_short_edge: 156.0,
            maximum_long_edge: 560.0,
            caption: 0.0,
            pixel_bound: true,
        },
    ),
    (
        "file",
        Footprint {
            preferred_long_edge: 272.0,
            minimum_short_edge: 150.0,
            maximum_long_edge: 480.0,
            caption: 48.0,
            pixel_bound: false,
        },
    ),
];

fn footprint(kind: &str) -> Option<&'static Footprint> {
    FOOTPRINTS
        .iter()
        .find(|(name, _)| *name == kind)
        .map(|(_, footprint)| footprint)
}

pub(in crate::spaces) async fn sync_changed_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    changes: &[NodeContentChange],
) -> Result<Option<SpaceEditSnapshotResponse>, AppError> {
    let node_ids = changes
        .iter()
        .filter_map(|change| {
            let after = display_extent(&change.data)?;
            (display_extent(&change.expected_content) != Some(after)).then(|| change.id.clone())
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
        structure::repository::save_in_transaction(tx, space_id, revision, &mut snapshot).await?;
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
    let mut payload_by_node_id = HashMap::new();
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
            payload_by_node_id.insert(
                node_id.clone(),
                serde_json::from_str(&encoded).map_err(db::json_error)?,
            );
        }
    }
    let structure_ids = snapshot
        .structure
        .iter()
        .filter_map(|node| match node {
            StructureNode::Content { id, node_id, .. }
                if payload_by_node_id.contains_key(node_id) =>
            {
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
        let data = &payload_by_node_id[*node_id];
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
    payload: &Value,
    current: &Size,
    sizing_mode: SizingMode,
) -> Option<Size> {
    let kind = payload.get("kind")?.as_str()?;
    let extent = display_extent(payload)?;
    let ratio = extent.width / extent.height;
    match sizing_mode {
        SizingMode::Content => Some(default_size(footprint(kind)?, &extent)),
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

/// Whether this content shows a picture of itself, so its footprint follows the picture's aspect.
pub(in crate::spaces) fn is_pictured(payload: &Value) -> bool {
    display_extent(payload).is_some()
}

fn default_size(footprint: &Footprint, extent: &Size) -> Size {
    let ratio = extent.width / extent.height;
    let long = footprint.preferred_long_edge;
    let (mut width, mut height) = if ratio >= 1.0 {
        (long, long / ratio)
    } else {
        (long * ratio, long)
    };
    let short_edge = width.min(height);
    let long_edge = width.max(height);
    let mut scale = (footprint.minimum_short_edge / short_edge)
        .max(1.0)
        .min(footprint.maximum_long_edge / long_edge);
    if footprint.pixel_bound {
        scale = scale.min(extent.width / width);
    }
    width *= scale;
    height *= scale;
    Size {
        width: width.round(),
        height: (height + footprint.caption).round(),
    }
}

/// The size the content shows itself at in its own units: an image's cropped, rotated pixels, a
/// video's frame, a rendered document's page.
fn display_extent(payload: &Value) -> Option<Size> {
    let kind = payload.get("kind")?.as_str()?;
    if footprint(kind).is_none() {
        return None;
    }
    let intrinsic_width = positive_number(payload.get("intrinsicWidth")?)?;
    let intrinsic_height = positive_number(payload.get("intrinsicHeight")?)?;
    let extent = if kind == "image" {
        edited_extent(payload, intrinsic_width, intrinsic_height)
    } else {
        Size {
            width: intrinsic_width,
            height: intrinsic_height,
        }
    };
    let valid = [extent.width, extent.height]
        .iter()
        .all(|edge| edge.is_finite() && *edge > 0.0);
    valid.then_some(extent)
}

fn edited_extent(payload: &Value, intrinsic_width: f64, intrinsic_height: f64) -> Size {
    let edit = payload.get("imageEdit").filter(|value| value.is_object());
    let crop = edit.and_then(|value| value.get("crop"));
    let share = |axis: &str| {
        crop.and_then(|value| value.get(axis))
            .and_then(positive_number)
            .unwrap_or(1.0)
    };
    let width = intrinsic_width * share("width");
    let height = intrinsic_height * share("height");
    let radians = edit
        .and_then(|value| value.get("rotation"))
        .and_then(Value::as_f64)
        .unwrap_or(0.0)
        .to_radians();
    let (cosine, sine) = (radians.cos().abs(), radians.sin().abs());
    Size {
        width: width * cosine + height * sine,
        height: width * sine + height * cosine,
    }
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
