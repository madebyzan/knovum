use super::session;
use crate::cancellation::Cancellation;
use ort::{session::Session, value::Tensor};
use std::path::Path;

const TILE: usize = 128;
const CONTEXT: usize = 32;
const CORE: usize = 64;

pub(super) struct Upscaling {
    session: Session,
}

impl Upscaling {
    pub(super) fn new(directory: &Path, quality: bool, threads: u8) -> Result<Self, String> {
        let name = if quality {
            "real_esrgan_x4plus.onnx"
        } else {
            "real_esrgan_general_x4v3.onnx"
        };
        Ok(Self {
            session: session::load(
                &directory.join(name),
                threads,
                &["image"],
                &["upscaled_image"],
            )?,
        })
    }

    pub(super) fn execute(
        &mut self,
        shape: [u32; 4],
        source: &[u8],
        cancellation: &Cancellation,
    ) -> Result<Vec<u8>, String> {
        let [batch, channels, height, width] = shape;
        let (height, width) = (height as usize, width as usize);
        if batch != 1 || channels != 3 || height * width > 4_194_304 {
            return Err("invalid_input: upscale source exceeds tile budget".into());
        }
        let mut output = vec![0u8; 3 * height * width * 16];
        for top in (0..height).step_by(CORE) {
            for left in (0..width).step_by(CORE) {
                let input = tile(source, height, width, top, left);
                let input = Tensor::from_array(([1usize, 3, TILE, TILE], input))
                    .map_err(|error| format!("invalid_input: {error}"))?;
                let options = cancellation.run_options()?;
                let results = self
                    .session
                    .run_with_options(ort::inputs!["image" => input], &options)
                    .map_err(|error| format!("invalid_output: {error}"))?;
                cancellation.check()?;
                let (actual, pixels) = results["upscaled_image"]
                    .try_extract_tensor::<f32>()
                    .map_err(|error| format!("invalid_output: {error}"))?;
                if actual.as_ref() != [1, 3, 512, 512]
                    || pixels.iter().any(|value| !value.is_finite())
                {
                    return Err("invalid_output: invalid upscale tile".into());
                }
                for channel in 0..3 {
                    for row in 0..(height - top).min(CORE) * 4 {
                        for column in 0..(width - left).min(CORE) * 4 {
                            let value = pixels
                                [(channel * 512 + row + CONTEXT * 4) * 512 + column + CONTEXT * 4];
                            output[(channel * height * 4 + top * 4 + row) * width * 4
                                + left * 4
                                + column] = quantize(value)?;
                        }
                    }
                }
            }
        }
        Ok(output)
    }
}

fn tile(source: &[u8], height: usize, width: usize, top: usize, left: usize) -> Vec<f32> {
    let mut tile = Vec::with_capacity(3 * TILE * TILE);
    for channel in 0..3 {
        for row in 0..TILE {
            let y = (top + row).saturating_sub(CONTEXT).min(height - 1);
            for column in 0..TILE {
                let x = (left + column).saturating_sub(CONTEXT).min(width - 1);
                tile.push(f32::from(source[(channel * height + y) * width + x]) / 255.0);
            }
        }
    }
    tile
}

pub(super) fn quantize(value: f32) -> Result<u8, String> {
    if !value.is_finite() {
        return Err("invalid_output: non-finite upscale pixel".into());
    }
    Ok((value * 255.0).round_ties_even().clamp(0.0, 255.0) as u8)
}
