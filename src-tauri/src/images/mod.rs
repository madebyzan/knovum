mod background_removal;
mod edited;

pub(crate) use background_removal::remove_background;
pub(crate) use edited::render_source_png;
pub(crate) use edited::validate_edit;

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
