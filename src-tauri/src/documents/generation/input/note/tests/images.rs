use std::io::Cursor;

use office_oxide::ir::{Element, ImageFormat};
use serde_json::json;

use super::source;
use crate::documents::generation::{
    document::ImageSource,
    input::{note::parse, DocumentFileSource, DocumentInput},
    DocumentFormat,
};

#[test]
fn text_image_uses_alt_text_without_reading_a_snapshot() {
    let source = source(json!({"type":"doc","content":[{"type":"image","attrs":{
        "src":"asset://private-id","assetPath":"/private/path.png","alt":"Diagram","title":null,"width":null,"height":null
    }}]}));
    let ir = parse(&source, DocumentFormat::Text).unwrap();
    let Element::Paragraph(paragraph) = &ir.sections[0].elements[0] else {
        panic!()
    };
    assert_eq!(
        office_oxide::ir::inline_to_text(&paragraph.content),
        "[Image: Diagram]"
    );
    assert!(!ir.plain_text().contains("private"));
}

#[test]
fn rich_image_resolves_exact_source_and_embeds_normalized_png() {
    let mut png = Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(2, 3)
        .write_to(&mut png, image::ImageFormat::Png)
        .unwrap();
    let fixture = ImageFixture::new(png.into_inner());
    let mut source = source(json!({"type":"doc","content":[{"type":"image","attrs":{
        "src":"asset://image-1","assetPath":"/authorized/image.png","alt":"Chart","title":null,"width":null,"height":null
    }}]}));
    source.images.push(ImageSource {
        destination: "asset://image-1".into(),
        file: fixture.source.clone(),
    });
    let ir = parse(&source, DocumentFormat::Html).unwrap();
    let Element::Image(image) = &ir.sections[0].elements[0] else {
        panic!()
    };
    assert_eq!(image.alt_text.as_deref(), Some("Chart"));
    assert_eq!(image.format, Some(ImageFormat::Png));
    assert_eq!((image.pixel_width, image.pixel_height), (Some(2), Some(3)));
    assert!(image
        .data
        .as_ref()
        .is_some_and(|bytes| bytes.starts_with(b"\x89PNG")));
}

#[test]
fn note_docx_images_fit_the_page_and_keep_positive_dimensions() {
    for (width, height, requested_width) in [(1600, 1200, None), (1600, 1, Some(1))] {
        let mut png = Cursor::new(Vec::new());
        image::DynamicImage::new_rgb8(width, height)
            .write_to(&mut png, image::ImageFormat::Png)
            .unwrap();
        let fixture = ImageFixture::new(png.into_inner());
        let mut source = source(json!({"type":"doc","content":[{"type":"image","attrs":{
            "src":"asset://image-1","assetPath":"/authorized/image.png","alt":"Photo",
            "width":requested_width,"height":null
        }}]}));
        source.images.push(ImageSource {
            destination: "asset://image-1".into(),
            file: fixture.source.clone(),
        });
        let mut ir = parse(&source, DocumentFormat::Docx).unwrap();
        crate::documents::generation::docx::image_layout::prepare(&mut ir).unwrap();
        let Element::Image(image) = &ir.sections[0].elements[0] else {
            panic!()
        };
        assert!((1..=546 * 9_525).contains(&image.display_width_emu.unwrap()));
        assert!((1..=850 * 9_525).contains(&image.display_height_emu.unwrap()));
        assert_eq!(
            (image.pixel_width, image.pixel_height),
            (Some(width), Some(height))
        );
        if requested_width.is_none() {
            assert_eq!(image.display_width_emu, Some(546 * 9_525));
        }
        let size = (
            image.display_width_emu.unwrap(),
            image.display_height_emu.unwrap(),
        );
        let bytes = DocumentInput::Note(source)
            .render(DocumentFormat::Docx)
            .unwrap();
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut archive.by_name("word/document.xml").unwrap(), &mut xml)
            .unwrap();
        assert!(xml.contains(&format!("cx=\"{}\"", size.0)));
        assert!(xml.contains(&format!("cy=\"{}\"", size.1)));
    }
}

struct ImageFixture {
    source: DocumentFileSource,
}

impl ImageFixture {
    fn new(bytes: Vec<u8>) -> Self {
        let path =
            std::env::temp_dir().join(format!("knovum-note-image-{}.png", uuid::Uuid::new_v4()));
        std::fs::write(&path, bytes).unwrap();
        Self {
            source: DocumentFileSource {
                title: String::new(),
                source_revision: crate::file_inspection::content_revision(&path).unwrap(),
                path,
            },
        }
    }
}

impl Drop for ImageFixture {
    fn drop(&mut self) {
        std::fs::remove_file(&self.source.path).unwrap();
    }
}
