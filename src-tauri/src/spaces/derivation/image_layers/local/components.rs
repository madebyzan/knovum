//! Connected foreground components and their reading order. Used by flat-sheet discovery, where
//! each component is one subject on the sheet.

/// Smallest share of the canvas a component must cover to count as a subject.
const MIN_COMPONENT_RATIO: f32 = 0.001;
/// Share of the smaller bounding box that must sit inside a larger one before the two components
/// are treated as one subject. Detached parts such as dots or highlights land here.
const BBOX_CONTAINMENT: f32 = 0.9;

pub(super) struct Component {
    pub(super) pixels: Vec<u32>,
    /// Inclusive `[left, top, right, bottom]` bounds.
    pub(super) bounds: [u32; 4],
}

/// Labels connected foreground runs with an explicit stack, largest component first.
pub(super) fn find(mask: &[bool], width: u32, height: u32) -> Vec<Component> {
    let minimum = (mask.len() as f32 * MIN_COMPONENT_RATIO).ceil() as usize;
    let mut visited = vec![false; mask.len()];
    let mut found = Vec::new();
    for start in 0..mask.len() {
        if !mask[start] || visited[start] {
            continue;
        }
        visited[start] = true;
        let component = grow(start as u32, mask, &mut visited, (width, height));
        if component.pixels.len() >= minimum {
            found.push(component);
        }
    }
    found.sort_by(|left, right| right.pixels.len().cmp(&left.pixels.len()));
    found
}

fn grow(start: u32, mask: &[bool], visited: &mut [bool], (width, height): (u32, u32)) -> Component {
    let mut stack = vec![start];
    let mut component = Component {
        pixels: Vec::new(),
        bounds: [width, height, 0, 0],
    };
    while let Some(index) = stack.pop() {
        let (x, y) = (index % width, index / width);
        component.bounds = [
            component.bounds[0].min(x),
            component.bounds[1].min(y),
            component.bounds[2].max(x),
            component.bounds[3].max(y),
        ];
        component.pixels.push(index);
        for (nx, ny) in neighbors(x, y, width, height) {
            let neighbor = (ny * width + nx) as usize;
            if mask[neighbor] && !visited[neighbor] {
                visited[neighbor] = true;
                stack.push(neighbor as u32);
            }
        }
    }
    component
}

fn neighbors(x: u32, y: u32, width: u32, height: u32) -> impl Iterator<Item = (u32, u32)> {
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
        (nx < width && ny < height).then_some((nx, ny))
    })
}

/// Folds components whose bounding box sits inside a larger one into that subject.
pub(super) fn merge(components: Vec<Component>) -> Vec<Component> {
    let mut merged: Vec<Component> = Vec::new();
    for component in components {
        match merged
            .iter_mut()
            .find(|subject| contains(subject.bounds, component.bounds))
        {
            Some(subject) => subject.pixels.extend(component.pixels),
            None => merged.push(component),
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
    area > 0 && shared as f32 / area as f32 >= BBOX_CONTAINMENT
}

/// Orders components row by row. A component opens a new row once it starts below everything the
/// current row covers, which keeps a grid of subjects in the order a reader expects.
pub(super) fn reading_order(mut components: Vec<Component>) -> Vec<Component> {
    components.sort_by_key(|component| component.bounds[1]);
    let mut ordered: Vec<Component> = Vec::with_capacity(components.len());
    let mut row: Vec<Component> = Vec::new();
    let mut bottom = 0;
    for component in components {
        if !row.is_empty() && component.bounds[1] > bottom {
            row.sort_by_key(|member| member.bounds[0]);
            ordered.append(&mut row);
        }
        bottom = if row.is_empty() {
            component.bounds[3]
        } else {
            bottom.max(component.bounds[3])
        };
        row.push(component);
    }
    row.sort_by_key(|member| member.bounds[0]);
    ordered.append(&mut row);
    ordered
}

#[cfg(test)]
#[path = "tests/components.rs"]
mod tests;
