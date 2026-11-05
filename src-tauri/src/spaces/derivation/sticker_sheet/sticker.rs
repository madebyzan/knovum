//! One sticker as the WeChat sticker album asks for it: a square main image of `EDGE` pixels, the
//! sticker on a transparent ground and never touching the frame, well inside the size limit.

use image::{
    imageops::{self, FilterType},
    ImageBuffer, Rgba, RgbaImage,
};
use std::io::Cursor;

/// Edge of a WeChat sticker album main image.
pub(super) const EDGE: u32 = 240;
/// Clear space kept between the sticker and its frame: a sticker must not touch the edge.
pub(super) const MARGIN: u32 = 8;
/// Largest main image the album accepts.
const MAX_BYTES: usize = 500 * 1024;
// A frame of raw RGBA plus the PNG row filters, stored without any compression at all, still fits
// the album's size limit; so every sticker does, whatever it shows.
const _: () = assert!(((EDGE * 4 + 1) * EDGE) as usize + 4096 < MAX_BYTES);

/// Cuts the sticker at `bounds` (inclusive) out of the lifted sheet and fits it, centred and
/// aspect kept, inside the frame's clear space.
pub(super) fn frame(sheet: &RgbaImage, [left, top, right, bottom]: [u32; 4]) -> RgbaImage {
    let (width, height) = (right - left + 1, bottom - top + 1);
    let room = (EDGE - 2 * MARGIN) as f32;
    let scale = (room / width as f32).min(room / height as f32);
    let fitted = (
        ((width as f32 * scale).round() as u32).clamp(1, EDGE - 2 * MARGIN),
        ((height as f32 * scale).round() as u32).clamp(1, EDGE - 2 * MARGIN),
    );
    let cut = imageops::crop_imm(sheet, left, top, width, height).to_image();
    let resized = resize_premultiplied(&cut, fitted);
    let mut framed = RgbaImage::from_pixel(EDGE, EDGE, Rgba([0, 0, 0, 0]));
    imageops::replace(
        &mut framed,
        &resized,
        i64::from((EDGE - fitted.0) / 2),
        i64::from((EDGE - fitted.1) / 2),
    );
    framed
}

/// Resamples with colour weighted by coverage, so the transparent ground's colour never bleeds
/// into the sticker's edge as a dark or light fringe.
fn resize_premultiplied(image: &RgbaImage, (width, height): (u32, u32)) -> RgbaImage {
    let premultiplied =
        ImageBuffer::<Rgba<f32>, Vec<f32>>::from_fn(image.width(), image.height(), |x, y| {
            let pixel = image.get_pixel(x, y);
            let alpha = f32::from(pixel[3]) / 255.0;
            Rgba([0, 1, 2, 3].map(|channel| {
                let value = f32::from(pixel[channel]) / 255.0;
                if channel == 3 {
                    value
                } else {
                    value * alpha
                }
            }))
        });
    let resized = imageops::resize(&premultiplied, width, height, FilterType::Lanczos3);
    RgbaImage::from_fn(width, height, |x, y| {
        let pixel = resized.get_pixel(x, y);
        let alpha = pixel[3].clamp(0.0, 1.0);
        let byte = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
        if alpha <= f32::EPSILON {
            return Rgba([0, 0, 0, 0]);
        }
        Rgba([
            byte(pixel[0] / alpha),
            byte(pixel[1] / alpha),
            byte(pixel[2] / alpha),
            byte(alpha),
        ])
    })
}

pub(super) fn png(sticker: &RgbaImage) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    sticker
        .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
        .map_err(|error| error.to_string())?;
    Ok(bytes)
}

#[cfg(test)]
#[path = "tests/sticker.rs"]
mod tests;
