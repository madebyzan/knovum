use std::path::Path;

use crate::file_inspection::{classify_file, is_sensitive_file, ResourceClassification};

/// Quick Look and Windows Shell choose a renderer from the real file name and extension.
pub(crate) fn supports(source: &Path) -> bool {
    source.is_file() && renderable(source)
}

/// Source code owns a text preview. Native media, images and documents use the operating system's
/// registered thumbnail provider.
pub(super) fn renderable(source: &Path) -> bool {
    if is_sensitive_file(source) {
        return false;
    }
    matches!(
        classify_file(source),
        ResourceClassification::File
            | ResourceClassification::TextDocument
            | ResourceClassification::Image
            | ResourceClassification::Video
            | ResourceClassification::Audio
    )
}
