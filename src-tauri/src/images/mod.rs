mod background_removal;
mod edited;

pub(crate) use background_removal::pixels::{
    model_input as saliency_input, MODEL_RESOLUTION as SALIENCY_EDGE,
};
pub(crate) use background_removal::remove_background;
pub(crate) use edited::render_source_png;
pub(crate) use edited::render_source_png_within;
pub(crate) use edited::validate_edit;

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
