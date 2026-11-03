use super::*;

#[test]
fn transparent_pixels_read_as_paper() {
    let mut image = RgbaImage::new(1, 1);
    image.put_pixel(0, 0, image::Rgba([0, 0, 0, 0]));
    assert_eq!(
        on_paper(DynamicImage::ImageRgba8(image)).get_pixel(0, 0).0,
        [255; 4]
    );
}

#[test]
fn a_document_picture_names_its_resource_and_a_shown_picture_its_node() {
    let resource = RecognitionSource {
        picture: Picture::Resource {
            id: "scan".into(),
            path: "/assets/scan.png".into(),
        },
        kind: "page",
        location: json!({ "kind": "pdf", "page": 3 }),
    };
    let shown = RecognitionSource {
        picture: Picture::Shown,
        kind: "image_description",
        location: json!({ "kind": "node" }),
    };
    let [page] = passages(&resource, "Clause 4. Rent", "r1")
        .try_into()
        .unwrap();
    assert_eq!(
        page.metadata,
        Some(json!({ "role": "recognized_text", "resourceId": "scan" }))
    );
    assert_eq!(
        page.locator["location"],
        json!({ "kind": "pdf", "page": 3 })
    );
    assert_eq!(page.locator["sourceRevision"], "r1");
    let [photo] = passages(&shown, "Mill Lane", "r1").try_into().unwrap();
    assert_eq!(photo.metadata, Some(json!({ "role": "recognized_text" })));
    assert!(
        passages(&shown, "  ", "r1").is_empty(),
        "nothing read, nothing kept"
    );
}
