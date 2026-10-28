//! Table packing shared by every arrangement that lays roots out in reading order: a row, a
//! column or Tidy up of a selection, and the children of an auto-layout Section.

use std::collections::HashMap;

use crate::spaces::structure::planner::bounds::Bounds;
use crate::spaces::structure::planner::movement::{moved_layout, replace_layouts};
use crate::spaces::structure::planner::tree_policy::Index;
use crate::spaces::structure::repository::SpaceStructureSnapshot;

/// Offsets below this are float noise from re-deriving a layout that is already in place;
/// writing them would churn every row and history entry of a settled Section.
const SETTLED: f64 = 1e-6;

/// Rows top to bottom, each left to right. A root joins a row while its vertical centre lies in
/// the band of the row's topmost root, so a root dropped between two others lands between them
/// and a packed table reads back in the order it was packed.
pub(super) fn reading_order(bounds: &[Bounds]) -> Vec<usize> {
    let mut by_top: Vec<usize> = (0..bounds.len()).collect();
    by_top.sort_by(|a, b| {
        let (a, b) = (&bounds[*a], &bounds[*b]);
        a.top.total_cmp(&b.top).then(a.left.total_cmp(&b.left))
    });
    let mut rows: Vec<(f64, Vec<usize>)> = Vec::new();
    for root in by_top {
        let center = (bounds[root].top + bounds[root].bottom) / 2.0;
        match rows.last_mut() {
            Some((band_bottom, row)) if center < *band_bottom => row.push(root),
            _ => rows.push((bounds[root].bottom, vec![root])),
        }
    }
    rows.into_iter()
        .flat_map(|(_, mut row)| {
            row.sort_by(|a, b| bounds[*a].left.total_cmp(&bounds[*b].left));
            row
        })
        .collect()
}

/// Roots placed row-major into `columns` columns. Each column is as wide as its widest root and
/// each row as tall as its tallest, so columns stay aligned whatever the roots' sizes.
pub(super) struct Table {
    pub cells: Vec<(f64, f64)>,
    pub width: f64,
    pub height: f64,
}

pub(super) fn table(sizes: &[(f64, f64)], columns: usize, gap: f64) -> Table {
    let columns = columns.clamp(1, sizes.len().max(1));
    let rows = sizes.len().div_ceil(columns);
    let mut widths = vec![0.0_f64; columns];
    let mut heights = vec![0.0_f64; rows];
    for (position, (width, height)) in sizes.iter().enumerate() {
        widths[position % columns] = widths[position % columns].max(*width);
        heights[position / columns] = heights[position / columns].max(*height);
    }
    let offsets = |extents: &[f64]| {
        extents
            .iter()
            .scan(0.0, |start, extent| {
                let offset = *start;
                *start += extent + gap;
                Some(offset)
            })
            .collect::<Vec<_>>()
    };
    let (x, y) = (offsets(&widths), offsets(&heights));
    let span = |extents: &[f64]| {
        extents.iter().sum::<f64>() + gap * extents.len().saturating_sub(1) as f64
    };
    Table {
        cells: (0..sizes.len())
            .map(|position| (x[position % columns], y[position / columns]))
            .collect(),
        width: span(&widths),
        height: span(&heights),
    }
}

/// The most columns whose table still fits `width`; a single column when nothing fits.
pub(super) fn widest_table_within(sizes: &[(f64, f64)], gap: f64, width: f64) -> Table {
    (2..=sizes.len())
        .rev()
        .map(|columns| table(sizes, columns, gap))
        .find(|candidate| candidate.width <= width + SETTLED)
        .unwrap_or_else(|| table(sizes, 1, gap))
}

/// The table whose longer side is shortest: the compact block Tidy up leaves behind.
pub(super) fn squarest_table(sizes: &[(f64, f64)], gap: f64) -> Table {
    (1..=sizes.len().max(1))
        .map(|columns| table(sizes, columns, gap))
        .min_by(|a, b| a.width.max(a.height).total_cmp(&b.width.max(b.height)))
        .expect("a table always has at least one column count")
}

/// Moves each root so its bounds start at its cell, `order[k]` taking cell `k` of `table`.
pub(super) fn place(
    snapshot: &mut SpaceStructureSnapshot,
    index: &Index,
    roots: &[String],
    bounds: &[Bounds],
    order: &[usize],
    table: &Table,
    origin: (f64, f64),
) -> Result<(), String> {
    let mut updates = HashMap::with_capacity(roots.len());
    for (cell, root) in table.cells.iter().zip(order) {
        let dx = origin.0 + cell.0 - bounds[*root].left;
        let dy = origin.1 + cell.1 - bounds[*root].top;
        if dx.abs() > SETTLED || dy.abs() > SETTLED {
            updates.insert(
                roots[*root].clone(),
                moved_layout(snapshot, index, &roots[*root], dx, dy)?,
            );
        }
    }
    replace_layouts(&mut snapshot.layouts, updates);
    Ok(())
}
