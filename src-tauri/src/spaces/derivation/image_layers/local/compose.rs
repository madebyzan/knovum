//! Turns a decomposition into layer PNGs. Each layer is cropped to the pixels it covers, so a
//! subject is published at its own size instead of inside a canvas-sized transparent frame, and a
//! layer that carries no content of its own is not published at all.

use super::Layers;
use image::{imageops, Rgba, RgbaImage};
use std::io::Cursor;
use tokio_util::sync::CancellationToken;

/// Each layer becomes one image node asset, so it stays inside the budget a node may carry.
const MAX_LAYER_BYTES: usize = 20 * 1024 * 1024;

pub(super) fn encode(
    source: &RgbaImage,
    layers: &Layers,
    cancellation: &CancellationToken,
) -> Result<Vec<Vec<u8>>, String> {
    let decomposition = &layers.decomposition;
    if decomposition.labels.len() != (decomposition.width as usize * decomposition.height as usize)
        || decomposition.width != source.width()
        || decomposition.height != source.height()
    {
        return Err("Layer decomposition does not match its source".into());
    }
    let mut encoded = Vec::with_capacity(usize::from(decomposition.count) + 1);
    if let Some(backdrop) = layers.backdrop.as_ref().and_then(content) {
        encoded.push(png(&backdrop, cancellation)?);
    }
    for label in 1..=decomposition.count {
        if let Some(layer) = content(&subject(source, layers, label)) {
            encoded.push(png(&layer, cancellation)?);
        }
    }
    Ok(encoded)
}

/// One subject cut out of the source. The source alpha is preserved so antialiased sticker edges
/// stay soft, and a label that carries a soft alpha uses it instead of its binary mask.
fn subject(source: &RgbaImage, layers: &Layers, label: u8) -> RgbaImage {
    let soft = layers
        .alpha
        .as_ref()
        .filter(|(owner, _)| *owner == label)
        .map(|(_, alpha)| alpha);
    let mut subject = source.clone();
    for (index, pixel) in subject.pixels_mut().enumerate() {
        let coverage = if layers.decomposition.labels[index] == label {
            soft.map_or(255, |alpha| alpha[index])
        } else {
            0
        };
        pixel[3] = (u16::from(pixel[3]) * u16::from(coverage) / 255) as u8;
    }
    subject
}

/// Crops a layer to the pixels it covers. A layer of one repeated pixel carries nothing of its
/// own — an empty cut-out, or a backdrop that is a flat fill — so it is not a layer.
fn content(layer: &RgbaImage) -> Option<RgbaImage> {
    let [left, top, right, bottom] = covered(layer)?;
    let cropped =
        imageops::crop_imm(layer, left, top, right - left + 1, bottom - top + 1).to_image();
    let first = *cropped.get_pixel(0, 0);
    cropped
        .pixels()
        .any(|pixel| *pixel != first)
        .then_some(cropped)
}

/// Inclusive `[left, top, right, bottom]` bounds of the pixels a layer covers, or `None` when it
/// covers none.
fn covered(layer: &RgbaImage) -> Option<[u32; 4]> {
    layer
        .enumerate_pixels()
        .filter(|(_, _, pixel)| pixel[3] > 0)
        .fold(None, |covered, (x, y, _)| {
            Some(match covered {
                Some([left, top, right, bottom]) => {
                    [left.min(x), top.min(y), right.max(x), bottom.max(y)]
                }
                None => [x, y, x, y],
            })
        })
}

fn png(layer: &RgbaImage, cancellation: &CancellationToken) -> Result<Vec<u8>, String> {
    if cancellation.is_cancelled() {
        return Err("Image layer split was cancelled".into());
    }
    let mut bytes = Vec::new();
    layer
        .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
        .map_err(|error| error.to_string())?;
    if bytes.len() > MAX_LAYER_BYTES {
        return Err("A generated layer exceeds 20 MiB".into());
    }
    Ok(bytes)
}

/// The source with every subject removed. This is the backdrop layer when no pass could complete
/// one: it keeps the holes the subjects left, which is an honest record of what was lifted.
pub(super) fn without_subjects(source: &RgbaImage, labels: &[u8]) -> RgbaImage {
    let mut backdrop = source.clone();
    for (pixel, label) in backdrop.pixels_mut().zip(labels) {
        if *label != 0 {
            *pixel = Rgba([0, 0, 0, 0]);
        }
    }
    backdrop
}

#[cfg(test)]
#[path = "tests/compose.rs"]
mod tests;
