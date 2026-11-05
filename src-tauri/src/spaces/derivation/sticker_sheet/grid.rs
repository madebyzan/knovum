//! Reads the grid a sticker sheet is laid out on. Rows are read across the whole sheet, then each
//! row is read into its own columns, so a last row holding fewer stickers is read as it is.
//!
//! Along one axis the foreground profile has blank lines wherever nothing is drawn: the gutters
//! between stickers, but also the gap between a character and the caption under it. The two are
//! told apart by what a grid is — evenly pitched cells separated by gutters of one width — so a
//! sheet is read as the most cells that fit an even pitch through clean, consistent gutters, and a
//! caption therefore stays with the character it belongs to.

/// Most rows a sheet is read as, and most stickers a row is read as.
pub(super) const MAX_SPLITS: u32 = 8;
/// A line whose foreground stays at or below this share of its length — and at least one pixel —
/// is blank: compression ringing or a stray dot does not close a gutter.
const BLANK_SHARE: f32 = 0.002;
/// A cut may sit this share of the cell pitch away from where an even grid puts it.
const CUT_SLACK: f32 = 0.25;
/// The narrowest gutter of a reading must be at least this share of its widest. A sheet lays its
/// gutters out evenly, and the gap under a character is narrower than the gap between two
/// stickers; a reading that mixes the two is not the sheet's grid.
const MIN_GUTTER_RATIO: f32 = 0.4;
/// Cells narrower than this many pixels are not stickers.
const MIN_PITCH: f32 = 8.0;

/// Half-open `[start, end)` span along one axis.
type Span = (u32, u32);

/// The content bounds, inclusive `[left, top, right, bottom]`, of every cell that holds a sticker,
/// in reading order.
pub(super) fn cells(mask: &[bool], (width, height): (u32, u32)) -> Vec<[u32; 4]> {
    let rows = (0..height)
        .map(|y| line(mask, width, (0..width).map(|x| (x, y))))
        .collect::<Vec<_>>();
    split(&rows, width)
        .into_iter()
        .flat_map(|(top, bottom)| {
            let columns = (0..width)
                .map(|x| line(mask, width, (top..bottom).map(move |y| (x, y))))
                .collect::<Vec<_>>();
            split(&columns, bottom - top)
                .into_iter()
                .filter_map(move |(left, right)| bounds(mask, width, [left, top, right, bottom]))
        })
        .collect()
}

fn line(mask: &[bool], width: u32, pixels: impl Iterator<Item = (u32, u32)>) -> u32 {
    pixels
        .filter(|(x, y)| mask[(y * width + x) as usize])
        .count() as u32
}

/// Splits one profile into the spans an even grid reads it as: the most cells whose cuts all land
/// in blank lines near an even pitch and whose gutters are consistent with each other. With no
/// such grid the content is one span.
fn split(profile: &[u32], across: u32) -> Vec<Span> {
    let limit = (across as f32 * BLANK_SHARE).ceil().max(1.0) as u32;
    let blank = profile
        .iter()
        .map(|count| *count <= limit)
        .collect::<Vec<_>>();
    let Some(first) = blank.iter().position(|blank| !blank) else {
        return Vec::new();
    };
    let last = blank.iter().rposition(|blank| !blank).unwrap_or(first);
    let content = (first as u32, last as u32 + 1);
    (2..=MAX_SPLITS)
        .rev()
        .find_map(|count| even(&blank, content, count))
        .unwrap_or_else(|| vec![content])
}

fn even(blank: &[bool], (start, end): Span, count: u32) -> Option<Vec<Span>> {
    let pitch = (end - start) as f32 / count as f32;
    if pitch < MIN_PITCH {
        return None;
    }
    let slack = (pitch * CUT_SLACK) as u32;
    let mut gutters: Vec<Span> = Vec::with_capacity(count as usize - 1);
    for index in 1..count {
        let nominal = (start as f32 + pitch * index as f32).round() as u32;
        let gutter = gutter_near(blank, nominal, slack)?;
        if gutters
            .last()
            .is_some_and(|previous| previous.0 >= gutter.0)
        {
            return None;
        }
        gutters.push(gutter);
    }
    let widths = gutters.iter().map(|(low, high)| high - low);
    let (narrowest, widest) = widths.fold((u32::MAX, 0), |(low, high), width| {
        (low.min(width), high.max(width))
    });
    if (narrowest as f32) < widest as f32 * MIN_GUTTER_RATIO {
        return None;
    }
    let mut spans = Vec::with_capacity(count as usize);
    let mut from = start;
    for (low, high) in gutters {
        spans.push((from, low));
        from = high;
    }
    spans.push((from, end));
    Some(spans)
}

/// The whole blank run holding the blank line nearest to `nominal`, searched within `slack`.
fn gutter_near(blank: &[bool], nominal: u32, slack: u32) -> Option<Span> {
    let hit = (0..=slack).find_map(|offset| {
        [nominal.checked_sub(offset), nominal.checked_add(offset)]
            .into_iter()
            .flatten()
            .find(|at| blank.get(*at as usize).copied().unwrap_or(false))
    })? as usize;
    let low = blank[..hit]
        .iter()
        .rposition(|blank| !blank)
        .map_or(0, |at| at + 1);
    let high = blank[hit..]
        .iter()
        .position(|blank| !blank)
        .map_or(blank.len(), |at| hit + at);
    Some((low as u32, high as u32))
}

/// Inclusive bounds of the foreground inside the half-open cell `[left, top, right, bottom)`.
fn bounds(mask: &[bool], width: u32, [left, top, right, bottom]: [u32; 4]) -> Option<[u32; 4]> {
    (top..bottom)
        .flat_map(|y| (left..right).map(move |x| (x, y)))
        .filter(|(x, y)| mask[(y * width + x) as usize])
        .fold(None, |found, (x, y)| {
            Some(found.map_or([x, y, x, y], |[l, t, r, b]: [u32; 4]| {
                [l.min(x), t.min(y), r.max(x), b.max(y)]
            }))
        })
}

#[cfg(test)]
#[path = "tests/grid.rs"]
mod tests;
