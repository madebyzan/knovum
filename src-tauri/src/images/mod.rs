mod cutout;
mod edited;
mod segmentation;

pub(crate) use cutout::cut_out;
pub(crate) use edited::render_source_as;
pub(crate) use edited::render_source_png;
pub(crate) use edited::render_source_png_within;
#[cfg(test)]
pub(crate) use edited::tests::quarter_turn as quarter_turn_edit;
pub(crate) use edited::validate_edit;
pub(crate) use edited::Encoding;
pub(crate) use segmentation::{remove_background, subject_coverage};

/// The engine never produces a PNG larger than this, whoever consumes it.
pub(crate) const MAX_PNG_BYTES: usize = 64 * 1024 * 1024;

pub(crate) struct BackgroundRemovalOutput {
    pub png: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub(crate) struct EditedImageOutput {
    pub png: Vec<u8>,
    pub width: u32,
    pub height: u32,
}
