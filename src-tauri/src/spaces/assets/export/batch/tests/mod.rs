use super::file_parts;

#[test]
fn member_names_cannot_leave_the_selected_directory() {
    for name in [
        "",
        ".",
        "..",
        "../picture.png",
        "folder/picture.png",
        "folder\\picture.png",
        "/picture.png",
        "picture",
    ] {
        assert!(file_parts(name).is_err(), "accepted {name}");
    }
}

#[test]
fn member_names_keep_the_requested_encoding_and_unicode_title() {
    assert_eq!(file_parts("画面.png").unwrap(), ("画面", "png"));
    assert_eq!(file_parts("draft.v2.md").unwrap(), ("draft.v2", "md"));
}
