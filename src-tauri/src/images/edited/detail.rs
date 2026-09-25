use image::RgbaImage;

use super::tone::to_byte;

pub(super) fn apply_detail(output: &mut RgbaImage, sharpness: f64, clarity: f64, grain: f64) {
    if sharpness == 0.0 && clarity == 0.0 && grain == 0.0 {
        return;
    }
    let (width, height) = output.dimensions();
    let strength = sharpness / 100.0 * 1.2;
    let mut previous: Option<Vec<[u8; 4]>> = None;
    let mut current = read_row(output, 0);
    for y in 0..height {
        let next = (y + 1 < height).then(|| read_row(output, y + 1));
        for x in 0..width as usize {
            let center = current[x];
            if center[3] == 0 {
                continue;
            }
            let mut sum = [0.0; 3];
            let mut count = 0.0;
            if x > 0 {
                add_neighbor(current[x - 1], &mut sum, &mut count);
            }
            if x + 1 < width as usize {
                add_neighbor(current[x + 1], &mut sum, &mut count);
            }
            if let Some(row) = &previous {
                add_neighbor(row[x], &mut sum, &mut count);
            }
            if let Some(row) = &next {
                add_neighbor(row[x], &mut sum, &mut count);
            }
            let luminance = 0.2126 * f64::from(center[0])
                + 0.7152 * f64::from(center[1])
                + 0.0722 * f64::from(center[2]);
            let neighbor_luminance = if count > 0.0 {
                0.2126 * sum[0] / count + 0.7152 * sum[1] / count + 0.0722 * sum[2] / count
            } else {
                luminance
            };
            let mid_weight = 1.0 - (luminance / 127.5 - 1.0).abs();
            let clarity_shift =
                (luminance - neighbor_luminance) * clarity / 100.0 * 0.8 * mid_weight;
            let hash = (x as u32).wrapping_mul(73_856_093) ^ y.wrapping_mul(19_349_663);
            let grain_shift = (f64::from(hash & 255) / 255.0 - 0.5) * grain / 100.0 * 38.0;
            let pixel = output.get_pixel_mut(x as u32, y);
            for channel in 0..3 {
                let original = f64::from(center[channel]);
                let mean = if count > 0.0 {
                    sum[channel] / count
                } else {
                    original
                };
                pixel[channel] =
                    to_byte(original + strength * (original - mean) + clarity_shift + grain_shift);
            }
        }
        if let Some(row) = next {
            previous = Some(current);
            current = row;
        }
    }
}

fn read_row(image: &RgbaImage, y: u32) -> Vec<[u8; 4]> {
    (0..image.width())
        .map(|x| image.get_pixel(x, y).0)
        .collect()
}

fn add_neighbor(pixel: [u8; 4], sum: &mut [f64; 3], count: &mut f64) {
    if pixel[3] == 0 {
        return;
    }
    *count += 1.0;
    for channel in 0..3 {
        sum[channel] += f64::from(pixel[channel]);
    }
}
