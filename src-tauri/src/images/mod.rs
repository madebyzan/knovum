mod background_removal;

pub(crate) use background_removal::remove_background;

pub(crate) struct BackgroundRemovalOutput {
    pub png: Vec<u8>,
    pub width: u32,
    pub height: u32,
}
