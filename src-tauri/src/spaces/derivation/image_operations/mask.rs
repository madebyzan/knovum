use std::io::Cursor;

use base64::{engine::general_purpose::STANDARD, Engine};
use image::{ImageFormat, Rgb, RgbImage, Rgba, RgbaImage};
use tokio_util::sync::CancellationToken;

use super::contracts::MaskStroke;

const MAX_MASK_PIXEL_VISITS: u64 = 200_000_000;

pub fn encode(
    width: u32,
    height: u32,
    strokes: &[MaskStroke],
    cancellation: &CancellationToken,
) -> Result<String, String> {
    let mask = rasterize(width, height, strokes, cancellation)?;
    let mut png = Vec::new();
    image::DynamicImage::ImageRgb8(mask)
        .write_to(&mut Cursor::new(&mut png), ImageFormat::Png)
        .map_err(|error| format!("Object removal mask could not be encoded: {error}"))?;
    if png.len() > 10 * 1024 * 1024 {
        return Err("Object removal mask exceeds the provider limit".into());
    }
    Ok(format!("data:image/png;base64,{}", STANDARD.encode(png)))
}

pub fn encode_openai(
    width: u32,
    height: u32,
    strokes: &[MaskStroke],
    cancellation: &CancellationToken,
) -> Result<Vec<u8>, String> {
    let selection = rasterize(width, height, strokes, cancellation)?;
    let mut mask = RgbaImage::new(width, height);
    for y in 0..height {
        if cancellation.is_cancelled() {
            return Err("Image operation was cancelled".into());
        }
        for x in 0..width {
            mask.put_pixel(x, y, Rgba([0, 0, 0, 255 - selection.get_pixel(x, y)[0]]));
        }
    }
    let mut png = Vec::new();
    image::DynamicImage::ImageRgba8(mask)
        .write_to(&mut Cursor::new(&mut png), ImageFormat::Png)
        .map_err(|error| format!("Object removal mask could not be encoded: {error}"))?;
    if png.len() > 10 * 1024 * 1024 {
        return Err("Object removal mask exceeds the provider limit".into());
    }
    Ok(png)
}

fn rasterize(
    width: u32,
    height: u32,
    strokes: &[MaskStroke],
    cancellation: &CancellationToken,
) -> Result<RgbImage, String> {
    if !(512..=4096).contains(&width) || !(512..=4096).contains(&height) {
        return Err(
            "Object removal requires an image between 512 and 4096 pixels on each side".into(),
        );
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
mod tests {
    use super::*;

    #[test]
    fn mask_marks_only_selected_pixels() {
        let stroke = MaskStroke {
            diameter: 0.05,
            points: vec![0.5, 0.5, 0.5, 0.5],
        };
        let uri = encode(512, 512, &[stroke], &CancellationToken::new()).unwrap();
        let bytes = STANDARD
            .decode(uri.strip_prefix("data:image/png;base64,").unwrap())
            .unwrap();
        let mask = image::load_from_memory(&bytes).unwrap().to_luma8();
        assert_eq!(mask.get_pixel(0, 0)[0], 0);
        assert_eq!(mask.get_pixel(256, 256)[0], 255);
    }

    #[test]
    fn openai_mask_makes_only_selected_pixels_transparent() {
        let stroke = MaskStroke {
            diameter: 0.05,
            points: vec![0.5, 0.5, 0.5, 0.5],
        };
        let bytes = encode_openai(512, 512, &[stroke], &CancellationToken::new()).unwrap();
        let mask = image::load_from_memory(&bytes).unwrap().to_rgba8();
        assert_eq!(mask.get_pixel(0, 0)[3], 255);
        assert_eq!(mask.get_pixel(256, 256)[3], 0);
    }

    #[test]
    fn cancelled_mask_does_not_continue_rasterizing() {
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let stroke = MaskStroke {
            diameter: 0.05,
            points: vec![0.0, 0.0, 1.0, 1.0],
        };
        assert!(encode(512, 512, &[stroke], &cancellation)
            .unwrap_err()
            .contains("cancelled"));
    }

    #[test]
    fn diagonal_stroke_marks_its_corridor_only() {
        let stroke = MaskStroke {
            diameter: 0.01,
            points: vec![0.0, 0.0, 1.0, 1.0],
        };
        let uri = encode(512, 512, &[stroke], &CancellationToken::new()).unwrap();
        let bytes = STANDARD
            .decode(uri.strip_prefix("data:image/png;base64,").unwrap())
            .unwrap();
        let mask = image::load_from_memory(&bytes).unwrap().to_luma8();
        assert_eq!(mask.get_pixel(256, 256)[0], 255);
        assert_eq!(mask.get_pixel(256, 0)[0], 0);
    }
}
