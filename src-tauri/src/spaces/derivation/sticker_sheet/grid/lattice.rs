//! A lattice of `count` evenly pitched cells over the blank runs of one profile.
//!
//! Cell edges sit at `x_k = x_0 + k · pitch`, `k = 0..=count`. Each edge is crossed by a cut lying
//! in a blank run — the first in the leading margin, the last in the trailing one — and a cell
//! whose two cuts share a run holds nothing. A sheet is laid out by eye or by a model, not to the
//! pixel, so a sticker may reach a little past its cell's edge: a cut lies within
//! `CELL_OVERHANG · pitch` of its edge. Which run each cut takes decides the reading; the pitch is
//! solved from the runs, never guessed from where the content starts and ends.
//!
//! A lattice may also cut through the gap between a character and its caption or prop when every
//! cell has one. Such a reading leaves cells holding part of a sticker, and a part is not
//! comparable in size to a whole sticker, so a reading is kept only when its cells are.

use super::Span;

/// Share of the pitch a sticker may reach past its cell's edge.
const CELL_OVERHANG: f64 = 0.05;
/// The smallest cell's drawn extent must be at least this share of the largest's. The stickers of
/// one sheet are drawn at comparable sizes; a caption or a prop alone is not.
const MIN_EXTENT_RATIO: f32 = 0.4;

/// Pitches still possible, as a closed interval.
type Pitch = (f64, f64);
/// The smallest and the largest drawn extent among the cells cut so far.
type Extents = (u32, u32);

pub(super) struct Lattice<'a> {
    runs: &'a [Span],
    count: usize,
}

impl<'a> Lattice<'a> {
    /// `runs` as `grid::blank_runs` reads them: leading margin first, trailing margin last.
    pub(super) fn new(runs: &'a [Span], count: u32) -> Self {
        Self {
            runs,
            count: count as usize,
        }
    }

    /// The drawn span of every cell that holds a sticker, when such a lattice fits.
    pub(super) fn read(&self) -> Option<Vec<Span>> {
        let mut taken = vec![0];
        self.descend(&mut taken, (0.0, f64::INFINITY), (u32::MAX, 0))?;
        Some(
            taken
                .windows(2)
                .filter_map(|pair| self.cell(pair[0], pair[1]))
                .collect(),
        )
    }

    /// Places every remaining cut, `taken` holding the run of each cut placed so far, and leaves
    /// the whole reading in `taken` once one is found. Runs are ordered, so the next cut is looked
    /// for only up to where the widest pitch left can reach.
    fn descend(&self, taken: &mut Vec<usize>, pitch: Pitch, extents: Extents) -> Option<()> {
        let placed = taken.len();
        if placed > self.count {
            return Some(());
        }
        let last = self.runs.len() - 1;
        let previous = taken[placed - 1];
        let first = if placed == self.count { last } else { previous };
        let reach = self.runs[previous].1 as f64 + pitch.1 * (1.0 + 2.0 * CELL_OVERHANG);
        (first..=last)
            .take_while(|run| self.runs[*run].0 as f64 <= reach)
            .find_map(|run| {
                let extents = self.widen(extents, previous, run)?;
                let pitch = self.narrow(taken, run, pitch)?;
                taken.push(run);
                let found = self.descend(taken, pitch, extents);
                if found.is_none() {
                    taken.pop();
                }
                found
            })
    }

    /// The pitches that keep every cut placed so far in its run once the next cut lies in `run`.
    /// Edge `x_k` lies in `[lo_k − o, hi_k + o]` with `o = CELL_OVERHANG · pitch`. One `x_0` meets
    /// all of them exactly when every pair of cuts `i < j` allows it:
    /// `lo_j − hi_i ≤ (j − i + 2·CELL_OVERHANG) · pitch` and
    /// `(j − i − 2·CELL_OVERHANG) · pitch ≤ hi_j − lo_i`, so what is left is one interval.
    fn narrow(&self, taken: &[usize], run: usize, (low, high): Pitch) -> Option<Pitch> {
        let (start, end) = self.runs[run];
        let at = taken.len();
        let (low, high) = (0..)
            .zip(taken)
            .fold((low, high), |(low, high), (index, earlier)| {
                let (earlier_start, earlier_end) = self.runs[*earlier];
                let steps = (at - index) as f64;
                let floor = (start as f64 - earlier_end as f64) / (steps + 2.0 * CELL_OVERHANG);
                let ceiling = (end as f64 - earlier_start as f64) / (steps - 2.0 * CELL_OVERHANG);
                (low.max(floor), high.min(ceiling))
            });
        (low <= high && high > 0.0).then_some((low, high))
    }

    /// Extents once the cell between cuts in `from` and `to` joins them, refused once they are no
    /// longer comparable — no later cell can make them so again.
    fn widen(&self, (smallest, largest): Extents, from: usize, to: usize) -> Option<Extents> {
        let Some((start, end)) = self.cell(from, to) else {
            return Some((smallest, largest));
        };
        let (smallest, largest) = (smallest.min(end - start), largest.max(end - start));
        (smallest as f32 >= largest as f32 * MIN_EXTENT_RATIO).then_some((smallest, largest))
    }

    /// The drawn span between cuts in two runs; nothing when they share one.
    fn cell(&self, from: usize, to: usize) -> Option<Span> {
        (from != to).then(|| (self.runs[from].1, self.runs[to].0))
    }
}
