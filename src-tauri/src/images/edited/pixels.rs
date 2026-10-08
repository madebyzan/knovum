use image::{Rgba, RgbaImage};

use super::{
    contracts::ImageEdit,
    geometry::{crop_contains, RenderGeometry},
    glow::apply_glow,
    texture::apply_texture,
    tone::{to_byte, ToneAdjuster},
};

pub(super) fn render(source: &RgbaImage, edit: &ImageEdit) -> Result<RgbaImage, String> {
    let geometry = RenderGeometry::new(source.width(), source.height(), edit)?;
    let tone = ToneAdjuster::new(&edit.adjustments);
    let mut output = RgbaImage::new(geometry.output_width, geometry.output_height);
    for y in 0..geometry.output_height {
        for x in 0..geometry.output_width {
            let Some((source_x, source_y)) = geometry.source_point(x, y) else {
                continue;
            };
            if !crop_contains(
                &edit.crop,
                source_x,
                source_y,
                source.width(),
                source.height(),
            ) {
                continue;
            }
            let mut pixel = sample_bilinear(source, source_x, source_y);
            tone.apply(
                &mut pixel,
                x,
                y,
                geometry.output_width,
                geometry.output_height,
            );
            output.put_pixel(x, y, pixel);
        }
    }
    apply_texture(
        &mut output,
        edit.adjustments.sharpness,
        edit.adjustments.clarity,
        edit.adjustments.grain,
    );
    apply_glow(&mut output, edit.adjustments.glow);
    Ok(output)
}

fn sample_bilinear(source: &RgbaImage, x: f64, y: f64) -> Rgba<u8> {
    let sample_x = x - 0.5;
    let sample_y = y - 0.5;
    let left = sample_x.floor() as i64;
    let top = sample_y.floor() as i64;
    let fraction_x = sample_x - left as f64;
    let fraction_y = sample_y - top as f64;
    let mut premultiplied = [0.0; 4];
    for (pixel_x, pixel_y, weight) in [
        (left, top, (1.0 - fraction_x) * (1.0 - fraction_y)),
        (left + 1, top, fraction_x * (1.0 - fraction_y)),
        (left, top + 1, (1.0 - fraction_x) * fraction_y),
        (left + 1, top + 1, fraction_x * fraction_y),
    ] {
        if pixel_x < 0
            || pixel_y < 0
            || pixel_x >= i64::from(source.width())
            || pixel_y >= i64::from(source.height())
        {
            continue;
        }
        let pixel = source.get_pixel(pixel_x as u32, pixel_y as u32).0;
        let alpha = f64::from(pixel[3]) / 255.0;
        for channel in 0..3 {
            premultiplied[channel] += f64::from(pixel[channel]) * alpha * weight;
        }
        premultiplied[3] += f64::from(pixel[3]) * weight;
    }
    if premultiplied[3] == 0.0 {
        return Rgba([0, 0, 0, 0]);
    }
    let alpha = premultiplied[3] / 255.0;
    Rgba([
        to_byte(premultiplied[0] / alpha),
        to_byte(premultiplied[1] / alpha),
        to_byte(premultiplied[2] / alpha),
        to_byte(premultiplied[3]),
    ])
}
