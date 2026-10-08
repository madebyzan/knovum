use super::contracts::{Crop, ImageEdit};

pub(super) struct RenderGeometry {
    pub source_width: f64,
    pub source_height: f64,
    pub crop_center_x: f64,
    pub crop_center_y: f64,
    pub output_width: u32,
    pub output_height: u32,
    cosine: f64,
    sine: f64,
    flip_x: f64,
    flip_y: f64,
}

impl RenderGeometry {
    pub(super) fn new(width: u32, height: u32, edit: &ImageEdit) -> Result<Self, String> {
        let source_width = f64::from(width) * edit.crop.width;
        let source_height = f64::from(height) * edit.crop.height;
        let radians = edit.rotation.to_radians();
        let cosine = normalized_trig(radians.cos());
        let sine = normalized_trig(radians.sin());
        let output_width = dimension(source_width * cosine.abs() + source_height * sine.abs())?;
        let output_height = dimension(source_width * sine.abs() + source_height * cosine.abs())?;
        validate_output(output_width, output_height)?;
        Ok(Self {
            source_width,
            source_height,
            crop_center_x: f64::from(width) * (edit.crop.x + edit.crop.width / 2.0),
            crop_center_y: f64::from(height) * (edit.crop.y + edit.crop.height / 2.0),
            output_width,
            output_height,
            cosine,
            sine,
            flip_x: if edit.flip_horizontal { -1.0 } else { 1.0 },
            flip_y: if edit.flip_vertical { -1.0 } else { 1.0 },
        })
    }

    pub(super) fn source_point(&self, x: u32, y: u32) -> Option<(f64, f64)> {
        let output_x = (f64::from(x) + 0.5 - f64::from(self.output_width) / 2.0) * self.flip_x;
        let output_y = (f64::from(y) + 0.5 - f64::from(self.output_height) / 2.0) * self.flip_y;
        let local_x = self.cosine * output_x + self.sine * output_y;
        let local_y = -self.sine * output_x + self.cosine * output_y;
        if local_x.abs() > self.source_width / 2.0 || local_y.abs() > self.source_height / 2.0 {
            return None;
        }
        Some((self.crop_center_x + local_x, self.crop_center_y + local_y))
    }
}

fn normalized_trig(value: f64) -> f64 {
    if value.abs() < 1e-12 {
        0.0
    } else if (value.abs() - 1.0).abs() < 1e-12 {
        value.signum()
    } else {
        value
    }
}

pub(super) fn crop_contains(crop: &Crop, x: f64, y: f64, width: u32, height: u32) -> bool {
    let left = crop.x * f64::from(width);
    let top = crop.y * f64::from(height);
    x >= left
        && y >= top
        && x <= left + crop.width * f64::from(width)
        && y <= top + crop.height * f64::from(height)
}

fn dimension(value: f64) -> Result<u32, String> {
    if value.is_finite() && value > 0.0 && value <= f64::from(u32::MAX) {
        Ok(value.ceil() as u32)
    } else {
        Err("Edited image dimensions are invalid".into())
    }
}

fn validate_output(width: u32, height: u32) -> Result<(), String> {
    const MAX_OUTPUT_DIMENSION: u32 = 15_000;
    const MAX_OUTPUT_PIXELS: u64 = 100_000_000;
    if width > MAX_OUTPUT_DIMENSION
        || height > MAX_OUTPUT_DIMENSION
        || u64::from(width) * u64::from(height) > MAX_OUTPUT_PIXELS
    {
        Err("Edited image exceeds the 100 megapixel output limit".into())
    } else {
        Ok(())
    }
}
