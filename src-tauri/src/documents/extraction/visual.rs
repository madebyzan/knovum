use std::io::Cursor;

use serde_json::Value;

pub(crate) const MIN_FIGURE_SIDE_PT: f64 = 32.0;
pub(crate) const MIN_FIGURE_AREA_PT2: f64 = 2_400.0;
pub(crate) const MIN_FIGURE_SIDE_PX: u32 = 32;
pub(crate) const MIN_FIGURE_AREA_PX2: u64 = 2_400;
const EMU_PER_POINT: f64 = 12_700.0;
const MIN_TILED_PER_CONTAINER: usize = 6;
const MIN_REPEATED_CONTAINERS: usize = 3;

/// A logical occurrence, not a standalone canvas node. Binary data never crosses IPC.
pub(crate) struct DocumentImage {
    pub(crate) id: String,
    pub(crate) location: Value,
    pub(crate) context: String,
    pub(crate) alt_text: Option<String>,
    pub(crate) data: Option<Vec<u8>>,
}

pub(crate) fn is_meaningful_point_size(width_pt: f64, height_pt: f64) -> bool {
    width_pt >= MIN_FIGURE_SIDE_PT
        && height_pt >= MIN_FIGURE_SIDE_PT
        && width_pt * height_pt >= MIN_FIGURE_AREA_PT2
}

pub(crate) fn is_meaningful_emu_size(width_emu: u64, height_emu: u64) -> bool {
    is_meaningful_point_size(
        width_emu as f64 / EMU_PER_POINT,
        height_emu as f64 / EMU_PER_POINT,
    )
}

pub(crate) fn is_meaningful_pixel_size(width_px: u32, height_px: u32) -> bool {
    width_px >= MIN_FIGURE_SIDE_PX
        && height_px >= MIN_FIGURE_SIDE_PX
        && u64::from(width_px) * u64::from(height_px) >= MIN_FIGURE_AREA_PX2
}

/// Checks encoded raster header dimensions without decoding pixel buffers.
/// Unrecognized or corrupt byte streams return `true` so downstream coverage still records why
/// a referenced image could not be decoded.
pub(crate) fn has_meaningful_raster_dimensions(bytes: &[u8]) -> bool {
    let Ok(format) = image::guess_format(bytes) else {
        return true;
    };
    let Ok((width, height)) =
        image::ImageReader::with_format(Cursor::new(bytes), format).into_dimensions()
    else {
        return true;
    };
    is_meaningful_pixel_size(width, height)
}

pub(crate) fn is_repeated_watermark_or_template(
    containers_with_item: usize,
    max_in_single_container: usize,
) -> bool {
    containers_with_item >= MIN_REPEATED_CONTAINERS
        || max_in_single_container >= MIN_TILED_PER_CONTAINER
        || (containers_with_item >= 2 && max_in_single_container >= 3)
}
