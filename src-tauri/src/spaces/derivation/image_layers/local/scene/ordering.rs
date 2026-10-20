//! Pure layer ordering. Discovery ranks subjects by mask confidence, which says nothing about
//! which subject sits behind which, so the stack order comes from the depth field.

use super::passes::Field;
use crate::spaces::derivation::image_layers::local::Decomposition;

/// Reorders discovery ranks into a far-to-near stack. Depth-Anything reports larger values for
/// nearer surfaces, so a subject with a smaller mean depth belongs further back. The depth field
/// keeps its own resolution, so ranks are sampled into it rather than upsampled.
pub(super) fn stack(
    ranks: Vec<u8>,
    (width, height): (u32, u32),
    depth: &Field,
) -> Result<Decomposition, String> {
    if ranks.len() != (width as usize * height as usize)
        || depth.values.len() != (depth.width as usize * depth.height as usize)
        || depth.width == 0
        || depth.height == 0
    {
        return Err("Depth field does not match the discovered subjects".into());
    }
    let count = ranks.iter().copied().max().unwrap_or(0);
    let mut sums = vec![(0.0_f64, 0_u64); usize::from(count) + 1];
    for (index, value) in depth.values.iter().enumerate() {
        let x = (index as u32 % depth.width) * width / depth.width;
        let y = (index as u32 / depth.width) * height / depth.height;
        let rank = ranks[(y.min(height - 1) * width + x.min(width - 1)) as usize];
        let entry = &mut sums[usize::from(rank)];
        entry.0 += f64::from(*value);
        entry.1 += 1;
    }
    let mut means = (1..=count)
        .map(|rank| {
            let (sum, area) = sums[usize::from(rank)];
            (
                rank,
                if area == 0 {
                    f64::MAX
                } else {
                    sum / area as f64
                },
            )
        })
        .collect::<Vec<_>>();
    means.sort_by(|left, right| left.1.total_cmp(&right.1));
    let mut order = vec![0_u8; usize::from(count) + 1];
    for (position, (rank, _)) in means.iter().enumerate() {
        order[usize::from(*rank)] = (position + 1) as u8;
    }
    Ok(Decomposition {
        labels: ranks.iter().map(|rank| order[usize::from(*rank)]).collect(),
        count,
        width,
        height,
    })
}

/// Picks the layer that holds the salient subject. Saliency has softer edges than a discovery
/// mask, so that layer uses it as its alpha.
pub(super) fn salient_layer(
    decomposition: &Decomposition,
    saliency: Vec<u8>,
) -> Option<(u8, Vec<u8>)> {
    if saliency.len() != decomposition.labels.len() {
        return None;
    }
    let owner = (1..=decomposition.count).max_by_key(|label| {
        decomposition
            .labels
            .iter()
            .zip(&saliency)
            .filter(|(at, _)| *at == label)
            .map(|(_, coverage)| u64::from(*coverage))
            .sum::<u64>()
    })?;
    Some((owner, saliency))
}

#[cfg(test)]
#[path = "tests/ordering.rs"]
mod tests;
