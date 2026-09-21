use super::web_image_extension;

#[test]
fn maps_standard_icon_media_types_to_ico() {
    for media_type in ["image/vnd.microsoft.icon", "image/x-icon"] {
        assert_eq!(
            web_image_extension(&media_type.parse().unwrap()).unwrap(),
            "ico"
        );
    }
}
