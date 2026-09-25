use image::{imageops, GrayImage, Luma, RgbaImage};

use super::tone::to_byte;

pub(super) fn apply_glow(output: &mut RgbaImage, amount: f64) {
    if amount == 0.0 {
        return;
    }
    let (width, height) = output.dimensions();
    let highlights = GrayImage::from_fn(width, height, |x, y| {
        let pixel = output.get_pixel(x, y);
        if pixel[3] == 0 {
            return Luma([0]);
        }
        let luminance = 0.2126 * f64::from(pixel[0])
            + 0.7152 * f64::from(pixel[1])
            + 0.0722 * f64::from(pixel[2]);
        Luma([to_byte((luminance - 140.0).max(0.0))])
    });
    let sigma = (width.max(height) as f32 / 300.0).max(1.0);
    let blurred = imageops::blur(&highlights, sigma);
    for y in 0..height {
        for x in 0..width {
            let pixel = output.get_pixel_mut(x, y);
            if pixel[3] == 0 {
                continue;
            }
            let lift = f64::from(blurred.get_pixel(x, y)[0]) / 255.0 * amount / 100.0 * 0.8;
            for channel in 0..3 {
                let value = f64::from(pixel[channel]);
                pixel[channel] = to_byte(value + (255.0 - value) * lift);
            }
        }
    }
}
