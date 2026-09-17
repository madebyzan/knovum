use image::ImageFormat;

use super::editor::{extension_for_format, format_for_media_type, validate_size};

#[test]
fn editor_image_policy_has_one_supported_format_mapping() {
    for (media_type, format, extension) in [
        ("image/png", ImageFormat::Png, "png"),
        ("image/jpeg", ImageFormat::Jpeg, "jpg"),
        ("image/gif", ImageFormat::Gif, "gif"),
        ("image/webp", ImageFormat::WebP, "webp"),
    ] {
        assert_eq!(format_for_media_type(media_type), Ok(format));
        assert_eq!(extension_for_format(format), extension);
    }
    assert!(format_for_media_type("image/svg+xml").is_err());
}

#[test]
fn editor_image_policy_enforces_the_single_size_limit() {
    assert!(validate_size(0).is_err());
    assert!(validate_size(1).is_ok());
    assert!(validate_size(20 * 1024 * 1024).is_ok());
    assert!(validate_size(20 * 1024 * 1024 + 1).is_err());
}
