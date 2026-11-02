use super::*;

#[test]
fn the_node_record_stops_at_its_budget_on_a_character_boundary() {
    let mut record = String::new();
    append_bounded(&mut record, &"页".repeat(MAX_TEXT - 1));
    append_bounded(
        &mut record,
        "ignored because the separator alone exhausts it",
    );
    assert!(record.encode_utf16().count() <= MAX_TEXT);
    assert!(record.starts_with('页'));
}

#[test]
fn pages_are_separated_while_the_budget_lasts() {
    let mut record = String::new();
    append_bounded(&mut record, "one");
    append_bounded(&mut record, "two");
    assert_eq!(record, "one\n\ntwo");
}

#[test]
fn transparent_pixels_read_as_paper() {
    let mut image = RgbaImage::new(1, 1);
    image.put_pixel(0, 0, image::Rgba([0, 0, 0, 0]));
    assert_eq!(
        on_paper(DynamicImage::ImageRgba8(image)).get_pixel(0, 0).0,
        [255; 4]
    );
}
