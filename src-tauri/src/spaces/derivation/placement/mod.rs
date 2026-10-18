//! Where derived content lands on the canvas.
//!
//! Every derivation - an Action's outputs, separated speech, an Agent's files - publishes through
//! here, so one rule covers all of them: a set published together is laid out as a grid and placed
//! as one block, and that block joins the field its sources' earlier derivations already form.

use serde_json::Value;
use sqlx::{Sqlite, Transaction};

use crate::foundation::app_error::AppError;
use crate::spaces::{
    db, nodes,
    structure::{
        self, DerivedBlock, Size, SizingMode, StructureLayout, StructureNode, CONTENT_GAP,
    },
};

/// Canvas footprint of content that cannot report a size yet.
pub(crate) const PLACEHOLDER: Size = Size {
    width: 320.0,
    height: 220.0,
};

/// A set of outputs published together.
pub(crate) struct DerivedOutputs<'a> {
    /// The nodes the outputs derive from, whose field they join.
    pub(crate) sources: &'a [String],
    /// Outputs already reserved on the canvas, which must not block their own set.
    pub(crate) reserved: &'a [String],
    /// Footprint of each output, in publication order.
    pub(crate) sizes: Vec<Size>,
}

/// Where one output sits, and the footprint it is published with.
pub(crate) struct DerivedCell {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) size: Size,
}

/// Plans a place for each output in the set.
pub(crate) async fn plan(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    structure: &[StructureNode],
    layouts: &[StructureLayout],
    outputs: DerivedOutputs<'_>,
) -> Result<Vec<DerivedCell>, AppError> {
    let grid = Grid::new(outputs.sizes)?;
    let siblings = field_size(tx, space_id, outputs.sources, outputs.reserved).await?;
    let (x, y) = structure::place_derived_content(
        structure,
        layouts,
        &DerivedBlock {
            source_node_ids: outputs.sources,
            placed_node_ids: outputs.reserved,
            siblings,
            width: grid.width,
            height: grid.height,
        },
    )?;
    Ok(grid
        .cells
        .into_iter()
        .map(|cell| DerivedCell {
            x: x + cell.x,
            y: y + cell.y,
            size: cell.size,
        })
        .collect())
}

/// The size the canvas gives this content, so the plan and the published node agree on one size.
pub(crate) fn fitted(data: &Value, requested: &Size, sizing_mode: SizingMode) -> Size {
    nodes::fitted_content_size(data, requested, sizing_mode).unwrap_or_else(|| requested.clone())
}

/// How many derived outputs the sources already carry. Outputs of this set are left out: they are
/// joining the field, not part of what it already holds.
async fn field_size(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    sources: &[String],
    reserved: &[String],
) -> Result<usize, AppError> {
    let mut members = std::collections::BTreeSet::new();
    for source in sources {
        let outputs: Vec<String> = sqlx::query_scalar(
            "SELECT relation.source_id FROM spaces_relations relation \
             JOIN spaces_nodes output ON output.id=relation.source_id \
             WHERE relation.space_id=$1 AND relation.target_id=$2 AND output.deleted_at IS NULL \
             AND relation.kind='derives-from' AND relation.status='active'",
        )
        .bind(space_id)
        .bind(source)
        .fetch_all(&mut **tx)
        .await
        .map_err(db::sql_error)?;
        members.extend(
            outputs
                .into_iter()
                .filter(|output| !reserved.contains(output)),
        );
    }
    Ok(members.len())
}

/// Where one output sits inside its set, relative to the block the set is placed at.
struct Cell {
    x: f64,
    y: f64,
    size: Size,
}

/// A set of outputs laid out as equal cells in reading order. Equal cells keep the set aligned
/// whatever each output's aspect is, the column count follows the square root so the set stays as
/// close to square as it can, and each output is centred in its cell.
struct Grid {
    width: f64,
    height: f64,
    cells: Vec<Cell>,
}

impl Grid {
    fn new(sizes: Vec<Size>) -> Result<Self, AppError> {
        let count = u32::try_from(sizes.len()).map_err(|_| "Too many derivation outputs")?;
        if count == 0 {
            return Err("A derivation publishes at least one output".into());
        }
        let cell = Size {
            width: sizes.iter().map(|size| size.width).fold(0.0, f64::max),
            height: sizes.iter().map(|size| size.height).fold(0.0, f64::max),
        };
        let columns = (1..=count)
            .find(|columns| columns * columns >= count)
            .unwrap_or(count);
        let mut cells = Vec::with_capacity(sizes.len());
        for (index, size) in sizes.into_iter().enumerate() {
            let index = u32::try_from(index).map_err(|_| "Too many derivation outputs")?;
            cells.push(Cell {
                x: f64::from(index % columns) * (cell.width + CONTENT_GAP)
                    + (cell.width - size.width) / 2.0,
                y: f64::from(index / columns) * (cell.height + CONTENT_GAP)
                    + (cell.height - size.height) / 2.0,
                size,
            });
        }
        Ok(Self {
            width: span(columns, cell.width),
            height: span(count.div_ceil(columns), cell.height),
            cells,
        })
    }
}

fn span(count: u32, cell: f64) -> f64 {
    f64::from(count) * cell + f64::from(count - 1) * CONTENT_GAP
}

#[cfg(test)]
mod tests;
