use image::{Rgb, RgbImage};
use tokio_util::sync::CancellationToken;

use super::contracts::MaskStroke;

const MAX_MASK_PIXEL_VISITS: u64 = 200_000_000;

pub(super) fn rasterize(
    width: u32,
    height: u32,
    strokes: &[MaskStroke],
    cancellation: &CancellationToken,
) -> Result<RgbImage, String> {
    if width == 0
        || height == 0
        || width > 16384
        || height > 16384
        || u64::from(width) * u64::from(height) > 16_777_216
    {
        return Err("Image mask exceeds the admitted pixel budget".into());
    }
    if cancellation.is_cancelled() {
        return Err("Image operation was cancelled".into());
    }
    let mut mask = RgbImage::from_pixel(width, height, Rgb([0, 0, 0]));
    let shortest = f64::from(width.min(height));
    let mut pixel_visits = 0;
    for stroke in strokes {
        if cancellation.is_cancelled() {
            return Err("Image operation was cancelled".into());
        }
        let radius = stroke.diameter * shortest / 2.0;
        let mut points = stroke.points.chunks_exact(2);
        let first = points.next().expect("validated mask stroke has a point");
        let mut start = (first[0] * f64::from(width), first[1] * f64::from(height));
        for point in points {
            let end = (point[0] * f64::from(width), point[1] * f64::from(height));
            paint_segment(
                &mut mask,
                start,
                end,
                radius,
                &mut pixel_visits,
                cancellation,
            )?;
            start = end;
        }
    }
    if cancellation.is_cancelled() {
        return Err("Image operation was cancelled".into());
    }
    Ok(mask)
}

fn paint_segment(
    mask: &mut RgbImage,
    start: (f64, f64),
    end: (f64, f64),
    radius: f64,
    pixel_visits: &mut u64,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let top = (start.1.min(end.1) - radius).floor().max(0.0) as u32;
    let bottom = (start.1.max(end.1) + radius)
        .ceil()
        .min(f64::from(mask.height() - 1)) as u32;
    let delta = (end.0 - start.0, end.1 - start.1);
    let length_squared = delta.0 * delta.0 + delta.1 * delta.1;
    for y in top..=bottom {
        if cancellation.is_cancelled() {
            return Err("Image operation was cancelled".into());
        }
        let center_y = f64::from(y) + 0.5;
        let (first_x, last_x) = if delta.1.abs() < f64::EPSILON {
            (start.0, end.0)
        } else {
            let first = ((center_y - radius - start.1) / delta.1).clamp(0.0, 1.0);
            let last = ((center_y + radius - start.1) / delta.1).clamp(0.0, 1.0);
            (start.0 + delta.0 * first, start.0 + delta.0 * last)
        };
        let left = (first_x.min(last_x) - radius).floor().max(0.0) as u32;
        let right = (first_x.max(last_x) + radius)
            .ceil()
            .min(f64::from(mask.width() - 1)) as u32;
        if left > right {
            continue;
        }
        *pixel_visits += u64::from(right - left + 1);
        if *pixel_visits > MAX_MASK_PIXEL_VISITS {
            return Err("Object removal selection is too complex; use fewer brush strokes".into());
        }
        for x in left..=right {
            let point = (f64::from(x) + 0.5, f64::from(y) + 0.5);
            let projection = if length_squared == 0.0 {
                0.0
            } else {
                (((point.0 - start.0) * delta.0 + (point.1 - start.1) * delta.1) / length_squared)
                    .clamp(0.0, 1.0)
            };
            let distance_squared = (point.0 - start.0 - projection * delta.0).powi(2)
                + (point.1 - start.1 - projection * delta.1).powi(2);
            if distance_squared <= radius * radius {
                mask.put_pixel(x, y, Rgb([255, 255, 255]));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
