mod background_removal;
mod edited;
pub(crate) mod matting;

pub(crate) use background_removal::pixels::{
    mask_at as saliency_mask, model_input_for_spec_id as saliency_input_for_spec,
};
pub(crate) use background_removal::remove_background;
pub(crate) use edited::render_source_as;
pub(crate) use edited::render_source_png;
pub(crate) use edited::render_source_png_within;
#[cfg(test)]
pub(crate) use edited::tests::quarter_turn as quarter_turn_edit;
pub(crate) use edited::validate_edit;
pub(crate) use edited::Encoding;

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
