/// The pixels an image operation works on: what the user currently sees,
/// already rendered by the engine within its own PNG ceiling.
pub struct SourceImage {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl From<crate::images::EditedImageOutput> for SourceImage {
    fn from(output: crate::images::EditedImageOutput) -> Self {
        Self {
            bytes: output.png,
            width: output.width,
            height: output.height,
        }
    }
}
