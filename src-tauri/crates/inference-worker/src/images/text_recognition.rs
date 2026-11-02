use crate::cancellation::Cancellation;
use image::RgbImage;
use knovum_inference_contracts::image_compute::RecognizedTextLine;
use oar_ocr::{core::config::OrtSessionConfig, oarocr::OAROCRBuilder, oarocr::OAROCR};
use std::path::Path;

pub(super) struct TextRecognition {
    /// Boxed: the pipeline is large and lives as long as the worker.
    engine: Box<OAROCR>,
}

impl TextRecognition {
    pub(super) fn new(directory: &Path, threads: u8) -> Result<Self, String> {
        let engine = OAROCRBuilder::new(
            directory.join("pp_ocrv6_small_det.onnx"),
            directory.join("pp_ocrv6_small_rec.onnx"),
            directory.join("ppocrv6_dict.txt"),
        )
        .ort_session(OrtSessionConfig::new().with_intra_threads(usize::from(threads)))
        .build()
        .map_err(|error| format!("runtime_mismatch: {error}"))?;
        Ok(Self {
            engine: Box::new(engine),
        })
    }

    /// The pipeline runs to completion once started, so one source is the unit of cancellation.
    pub(super) fn execute(
        &mut self,
        shape: [u32; 4],
        bytes: &[u8],
        cancellation: &Cancellation,
    ) -> Result<Vec<RecognizedTextLine>, String> {
        let source = interleaved(shape, bytes)?;
        cancellation.check()?;
        let result = self
            .engine
            .predict(vec![source])
            .map_err(|error| format!("invalid_output: {error}"))?;
        let page = result
            .into_iter()
            .next()
            .ok_or("invalid_output: text recognition returned no result")?;
        Ok(page.text_regions.iter().filter_map(line).collect())
    }
}

fn line(region: &oar_ocr::domain::TextRegion) -> Option<RecognizedTextLine> {
    let text = region.text.as_deref()?.trim();
    if text.is_empty() {
        return None;
    }
    let bounds = |axis: fn(&oar_ocr::processors::Point) -> f32| {
        region
            .bounding_box
            .points
            .iter()
            .map(axis)
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(low, high), value| {
                (low.min(value), high.max(value))
            })
    };
    let (left, right) = bounds(|point| point.x);
    let (top, bottom) = bounds(|point| point.y);
    (left.is_finite() && top.is_finite()).then(|| RecognizedTextLine {
        text: text.to_owned(),
        confidence: region.confidence.unwrap_or(0.0),
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    })
}

/// The wire carries planes like every other image tensor; the pipeline reads interleaved pixels.
pub(super) fn interleaved(shape: [u32; 4], planes: &[u8]) -> Result<RgbImage, String> {
    let [batch, channels, height, width] = shape;
    let count = u64::from(height) * u64::from(width);
    if batch != 1 || channels != 3 || count == 0 || planes.len() as u64 != count * 3 {
        return Err("invalid_input: text recognition takes one RGB tensor".into());
    }
    let count = count as usize;
    let (red, rest) = planes.split_at(count);
    let (green, blue) = rest.split_at(count);
    let pixels = red
        .iter()
        .zip(green)
        .zip(blue)
        .flat_map(|((red, green), blue)| [*red, *green, *blue])
        .collect();
    RgbImage::from_raw(width, height, pixels)
        .ok_or_else(|| "invalid_input: text recognition tensor differs from its shape".into())
}
