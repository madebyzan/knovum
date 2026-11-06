//! Where derived content lands on the canvas.
//!
//! Every derivation - an Action's outputs, separated speech, an Agent's files - publishes through
//! here, so one rule covers all of them: a set published together is laid out as a grid and placed
//! as one block, and that block joins the field its sources' earlier derivations already form.
//!
//! A set is read at one glance, so it claims the canvas a few outputs would rather than one full
//! footprint per member: up to `FULL_SIZE_MEMBERS` outputs keep the footprint each would have alone,
//! and a larger set keeps the area those would cover — its pictures and the gutters between them
//! shrink with the square root of its size, a picture down to `MEMBER_FLOOR` on its long edge.
//! A footprint the set chose is the node's own from then on (`SizingMode::Frame`), so content that
//! settles later keeps its place in the set rather than growing back to a lone output's size.
//! An output a person framed on the canvas before asking for it is the exception: it lands in its
//! frame, because that is where they were looking when they asked.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{Sqlite, Transaction};

use crate::foundation::app_error::AppError;
use crate::spaces::{
    db, nodes,
    structure::{
        self, DerivedBlock, Size, SizingMode, StructureLayout, StructureNode, CONTENT_GAP,
    },
};

/// How many outputs of one set keep the footprint each would have alone.
const FULL_SIZE_MEMBERS: f64 = 4.0;
/// The shortest long edge a picture in a set shrinks to; a smaller picture keeps its own size.
const MEMBER_FLOOR: f64 = 96.0;

/// Canvas footprint of content that cannot report a size yet.
pub(crate) const PLACEHOLDER: Size = Size {
    width: 320.0,
    height: 220.0,
};

/// Where a person framed an output before asking for it, in canvas coordinates.
#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct OutputFrame {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) width: f64,
    pub(crate) height: f64,
}

impl OutputFrame {
    pub(crate) fn validate(&self) -> Result<(), String> {
        let positioned = self.x.is_finite() && self.y.is_finite();
        let sized = [self.width, self.height]
            .iter()
            .all(|extent| extent.is_finite() && *extent > 0.0);
        if positioned && sized {
            Ok(())
        } else {
            Err(
                "canvas-placement-invalid: output frame must be a finite, non-empty rectangle"
                    .into(),
            )
        }
    }

    fn cell(&self) -> DerivedCell {
        DerivedCell {
            x: self.x,
            y: self.y,
            size: Size {
                width: self.width,
                height: self.height,
            },
            sizing_mode: SizingMode::Frame,
        }
    }
}

/// Where one reserved output goes.
pub(crate) enum OutputPlacement {
    /// Into its sources' derivation field, at this footprint until its content reports its own.
    Derived(Size),
    /// Exactly into the frame a person drew for it.
    Framed(OutputFrame),
}

/// Plans a cell for each output in order: a framed output takes its frame, and the rest are laid
/// out together as one block in their sources' field.
pub(crate) async fn place(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    structure: &[StructureNode],
    layouts: &[StructureLayout],
    sources: &[String],
    reserved: &[String],
    outputs: &[(&OutputPlacement, &Value)],
) -> Result<Vec<DerivedCell>, AppError> {
    let members: Vec<SetMember> = outputs
        .iter()
        .filter_map(|(placement, data)| match placement {
            OutputPlacement::Derived(footprint) => {
                Some(SetMember::new(data, footprint, SizingMode::Content))
            }
            OutputPlacement::Framed(_) => None,
        })
        .collect();
    let mut derived = if members.is_empty() {
        Vec::new()
    } else {
        plan(
            tx,
            space_id,
            structure,
            layouts,
            DerivedOutputs {
                sources,
                reserved,
                members,
            },
        )
        .await?
    }
    .into_iter();
    outputs
        .iter()
        .map(|(placement, _)| match placement {
            OutputPlacement::Framed(frame) => Ok(frame.cell()),
            OutputPlacement::Derived(_) => derived
                .next()
                .ok_or_else(|| AppError::from("A derived output has no planned place")),
        })
        .collect()
}

/// A set of outputs published together.
pub(crate) struct DerivedOutputs<'a> {
    /// The nodes the outputs derive from, whose field they join.
    pub(crate) sources: &'a [String],
    /// Outputs already reserved on the canvas, which must not block their own set.
    pub(crate) reserved: &'a [String],
    /// Each output, in publication order.
    pub(crate) members: Vec<SetMember>,
}

/// One output of a set: the footprint it would have alone, and how that footprint is kept.
#[derive(Clone)]
pub(crate) struct SetMember {
    size: Size,
    sizing_mode: SizingMode,
    pictured: bool,
}

impl SetMember {
    /// The output whose content is `data`, asked to be `requested` and kept by `sizing_mode`.
    pub(crate) fn new(data: &Value, requested: &Size, sizing_mode: SizingMode) -> Self {
        Self {
            size: fitted(data, requested, sizing_mode),
            sizing_mode,
            pictured: nodes::is_pictured(data),
        }
    }

    /// The footprint this member takes in a set scaled by `scale`. Only a picture shrinks: a
    /// document or a player keeps the size its text and controls need.
    fn in_set(self, scale: f64) -> Self {
        let long_edge = self.size.width.max(self.size.height);
        let scale = (scale * long_edge).max(MEMBER_FLOOR.min(long_edge)) / long_edge;
        if !self.pictured || scale >= 1.0 {
            return self;
        }
        Self {
            size: Size {
                width: (self.size.width * scale).round(),
                height: (self.size.height * scale).round(),
            },
            sizing_mode: SizingMode::Frame,
            pictured: true,
        }
    }
}

/// Where one output sits, and the footprint it is published with.
pub(crate) struct DerivedCell {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) size: Size,
    /// How the node keeps that footprint once published.
    pub(crate) sizing_mode: SizingMode,
}

/// Plans a place for each output in the set.
pub(crate) async fn plan(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    structure: &[StructureNode],
    layouts: &[StructureLayout],
    outputs: DerivedOutputs<'_>,
) -> Result<Vec<DerivedCell>, AppError> {
    let grid = Grid::new(outputs.members)?;
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
            size: cell.member.size,
            sizing_mode: cell.member.sizing_mode,
        })
        .collect())
}

/// The size the canvas gives this content, so the plan and the published node agree on one size.
fn fitted(data: &Value, requested: &Size, sizing_mode: SizingMode) -> Size {
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
    member: SetMember,
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
    fn new(members: Vec<SetMember>) -> Result<Self, AppError> {
        let count = u32::try_from(members.len()).map_err(|_| "Too many derivation outputs")?;
        if count == 0 {
            return Err("A derivation publishes at least one output".into());
        }
        let scale = (FULL_SIZE_MEMBERS / f64::from(count)).sqrt().min(1.0);
        let gap = (CONTENT_GAP * scale).round();
        let members: Vec<SetMember> = members
            .into_iter()
            .map(|member| member.in_set(scale))
            .collect();
        let cell = Size {
            width: members
                .iter()
                .map(|member| member.size.width)
                .fold(0.0, f64::max),
            height: members
                .iter()
                .map(|member| member.size.height)
                .fold(0.0, f64::max),
        };
        let columns = (1..=count)
            .find(|columns| columns * columns >= count)
            .unwrap_or(count);
        let mut cells = Vec::with_capacity(members.len());
        for (index, member) in (0..).zip(members) {
            cells.push(Cell {
                x: f64::from(index % columns) * (cell.width + gap)
                    + (cell.width - member.size.width) / 2.0,
                y: f64::from(index / columns) * (cell.height + gap)
                    + (cell.height - member.size.height) / 2.0,
                member,
            });
        }
        Ok(Self {
            width: span(columns, cell.width, gap),
            height: span(count.div_ceil(columns), cell.height, gap),
            cells,
        })
    }
}

fn span(count: u32, cell: f64, gap: f64) -> f64 {
    f64::from(count) * cell + f64::from(count - 1) * gap
}

#[cfg(test)]
mod tests;
