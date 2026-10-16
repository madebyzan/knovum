//! Turns a decomposition into stacked RGBA layers. Every layer keeps the source canvas size so
//! the stack composites back into the source without any placement metadata.

use super::Layers;
use image::{ImageFormat, Rgba, RgbaImage};
use std::io::Cursor;
use tokio_util::sync::CancellationToken;

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
    encoded.push(png(&backdrop(source, layers), cancellation)?);
    for label in 1..=decomposition.count {
        encoded.push(png(&subject(source, layers, label), cancellation)?);
    }
    Ok(encoded)
}

/// The bottom layer. A completed backdrop is used as is; without one the subjects leave holes,
/// which is the honest representation of what was removed.
fn backdrop(source: &RgbaImage, layers: &Layers) -> RgbaImage {
    if let Some(backdrop) = &layers.backdrop {
        return backdrop.clone();
    }
    let mut backdrop = source.clone();
    for (pixel, label) in backdrop.pixels_mut().zip(&layers.decomposition.labels) {
        if *label != 0 {
            *pixel = Rgba([0, 0, 0, 0]);
        }
    }
    backdrop
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

fn png(image: &RgbaImage, cancellation: &CancellationToken) -> Result<Vec<u8>, String> {
    if cancellation.is_cancelled() {
        return Err("Image layer split was cancelled".into());
    }
    let mut bytes = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .map_err(|error| error.to_string())?;
    if bytes.len() > MAX_LAYER_BYTES {
        return Err("A generated layer exceeds 20 MiB".into());
    }
    Ok(bytes)
}

#[cfg(test)]
#[path = "tests/compose.rs"]
mod tests;
