//! Pure binary-region geometry shared by the derivations that read structure from pixels: layer
//! splitting cleans the masks its models return, and sticker-sheet slicing reads its stickers off
//! the sheet. A mask is one `bool` per pixel in row-major order; every function here is total over
//! a mask that matches its dimensions and never allocates more than a few masks of its size.

/// One 8-connected run of mask pixels.
pub(crate) struct Region {
    /// Row-major pixel indices.
    pub(crate) pixels: Vec<u32>,
    /// Inclusive `[left, top, right, bottom]` bounds.
    pub(crate) bounds: [u32; 4],
}

/// Every 8-connected region of `mask`, largest first. Labelled with an explicit stack, so a region
/// the size of the canvas cannot overflow the call stack.
pub(crate) fn connected(mask: &[bool], (width, height): (u32, u32)) -> Vec<Region> {
    let mut visited = vec![false; mask.len()];
    let mut found = Vec::new();
    for start in 0..mask.len() {
        if !mask[start] || visited[start] {
            continue;
        }
        visited[start] = true;
        found.push(grow(start as u32, mask, &mut visited, (width, height)));
    }
    found.sort_by_key(|region| std::cmp::Reverse(region.pixels.len()));
    found
}

fn grow(start: u32, mask: &[bool], visited: &mut [bool], (width, height): (u32, u32)) -> Region {
    let mut stack = vec![start];
    let mut region = Region {
        pixels: Vec::new(),
        bounds: [width, height, 0, 0],
    };
    while let Some(index) = stack.pop() {
        let (x, y) = (index % width, index / width);
        region.bounds = [
            region.bounds[0].min(x),
            region.bounds[1].min(y),
            region.bounds[2].max(x),
            region.bounds[3].max(y),
        ];
        region.pixels.push(index);
        for neighbor in neighbors(x, y, (width, height)) {
            if mask[neighbor] && !visited[neighbor] {
                visited[neighbor] = true;
                stack.push(neighbor as u32);
            }
        }
    }
    region
}

/// Share of the smaller bounding box that must sit inside a larger one before the two regions are
/// one subject. Detached parts such as dots, highlights or an eye land here.
const BOUNDS_CONTAINMENT: f32 = 0.9;

/// Folds every region whose bounding box sits inside a larger one into that region. Regions arrive
/// largest first, so a part always meets the subject that holds it before it is kept on its own.
pub(crate) fn merge_contained(regions: Vec<Region>) -> Vec<Region> {
    let mut merged: Vec<Region> = Vec::new();
    for region in regions {
        match merged
            .iter_mut()
            .find(|subject| contains(subject.bounds, region.bounds))
        {
            Some(subject) => {
                subject.pixels.extend(region.pixels);
                subject.bounds = [
                    subject.bounds[0].min(region.bounds[0]),
                    subject.bounds[1].min(region.bounds[1]),
                    subject.bounds[2].max(region.bounds[2]),
                    subject.bounds[3].max(region.bounds[3]),
                ];
            }
            None => merged.push(region),
        }
    }
    merged
}

fn contains(subject: [u32; 4], part: [u32; 4]) -> bool {
    let overlap = |low: usize, high: usize| {
        (subject[high].min(part[high]) + 1).saturating_sub(subject[low].max(part[low]))
    };
    let shared = u64::from(overlap(0, 2)) * u64::from(overlap(1, 3));
    let area = u64::from(part[2] - part[0] + 1) * u64::from(part[3] - part[1] + 1);
    area > 0 && shared as f32 / area as f32 >= BOUNDS_CONTAINMENT
}

fn neighbors(x: u32, y: u32, (width, height): (u32, u32)) -> impl Iterator<Item = usize> {
    [
        (-1_i32, 0_i32),
        (1, 0),
        (0, -1),
        (0, 1),
        (-1, -1),
        (1, -1),
        (-1, 1),
        (1, 1),
    ]
    .into_iter()
    .filter_map(move |(dx, dy)| {
        let nx = x.checked_add_signed(dx)?;
        let ny = y.checked_add_signed(dy)?;
        (nx < width && ny < height).then_some((ny * width + nx) as usize)
    })
}

/// Pixels of `passable` that connect to the canvas edge through passable pixels. Reachability is
/// 4-connected: a one pixel diagonal gap in an outline is a closed outline, not a door.
pub(crate) fn reachable_from_edge(passable: &[bool], (width, height): (u32, u32)) -> Vec<bool> {
    let mut reached = vec![false; passable.len()];
    let mut stack = (0..passable.len())
        .filter(|&index| {
            let (x, y) = (index as u32 % width, index as u32 / width);
            (x == 0 || y == 0 || x + 1 == width || y + 1 == height) && passable[index]
        })
        .collect::<Vec<_>>();
    for &index in &stack {
        reached[index] = true;
    }
    while let Some(index) = stack.pop() {
        let (x, y) = (index as u32 % width, index as u32 / width);
        let sides = [
            (x > 0).then(|| index - 1),
            (x + 1 < width).then(|| index + 1),
            (y > 0).then(|| index - width as usize),
            (y + 1 < height).then(|| index + width as usize),
        ];
        for neighbor in sides.into_iter().flatten() {
            if passable[neighbor] && !reached[neighbor] {
                reached[neighbor] = true;
                stack.push(neighbor);
            }
        }
    }
    reached
}

/// Grows `mask` by `radius` pixels in every direction (a square structuring element). Separable
/// running counts keep the cost linear in the canvas whatever the radius.
pub(crate) fn dilate(mask: &[bool], (width, height): (u32, u32), radius: u32) -> Vec<bool> {
    if radius == 0 {
        return mask.to_vec();
    }
    let (width, height, radius) = (width as usize, height as usize, radius as usize);
    let mut rows = vec![false; mask.len()];
    for y in 0..height {
        let row = &mask[y * width..(y + 1) * width];
        sweep(row, &mut rows[y * width..(y + 1) * width], radius);
    }
    let mut grown = vec![false; mask.len()];
    let (mut column, mut swept) = (vec![false; height], vec![false; height]);
    for x in 0..width {
        for (y, cell) in column.iter_mut().enumerate() {
            *cell = rows[y * width + x];
        }
        sweep(&column, &mut swept, radius);
        for (y, cell) in swept.iter().enumerate() {
            grown[y * width + x] = *cell;
        }
    }
    grown
}

/// Shrinks `mask` by `radius` pixels: a pixel survives only when every pixel of its neighbourhood
/// on the canvas is set. The canvas edge is not a boundary — a subject the frame cuts through is
/// still solid where the frame cuts it.
pub(crate) fn erode(mask: &[bool], dimensions: (u32, u32), radius: u32) -> Vec<bool> {
    let outside = mask.iter().map(|inside| !inside).collect::<Vec<_>>();
    dilate(&outside, dimensions, radius)
        .into_iter()
        .map(|near_outside| !near_outside)
        .collect()
}

/// One line of a separable max filter: `out[i]` is set when any of `line[i-radius..=i+radius]` is.
fn sweep(line: &[bool], out: &mut [bool], radius: usize) {
    let mut prefix = Vec::with_capacity(line.len() + 1);
    prefix.push(0_u32);
    for value in line {
        prefix.push(prefix.last().copied().unwrap_or(0) + u32::from(*value));
    }
    for (index, slot) in out.iter_mut().enumerate() {
        let low = index.saturating_sub(radius);
        let high = (index + radius + 1).min(line.len());
        *slot = prefix[high] > prefix[low];
    }
}

#[cfg(test)]
mod tests;
