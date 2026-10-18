use std::{
    cell::Cell,
    collections::BTreeMap,
    io::{Cursor, Read},
};

use super::super::{
    document::{DocumentSource, ImageSource},
    images::{Image, Images},
    input::DocumentFileSource,
    DocumentFormat,
};

pub(super) struct ImageFixture(pub DocumentSource);

impl ImageFixture {
    pub fn new() -> Self {
        let mut png = Cursor::new(Vec::new());
        image::DynamicImage::new_rgb8(32, 16)
            .write_to(&mut png, image::ImageFormat::Png)
            .unwrap();
        let path = std::env::temp_dir().join(format!(
            "knovum-document-image-{}.png",
            uuid::Uuid::new_v4()
        ));
        std::fs::write(&path, png.into_inner()).unwrap();
        Self(DocumentSource {
            title: "图文文档".into(),
            markdown: "# 图文文档\n\n文字 **内容**\n\n![示意图](figure.png)".into(),
            citations: vec![],
            images: vec![ImageSource {
                destination: "figure.png".into(),
                file: DocumentFileSource {
                    title: String::new(),
                    source_revision: crate::file_inspection::content_revision(&path).unwrap(),
                    path,
                },
            }],
        })
    }
}

impl Drop for ImageFixture {
    fn drop(&mut self) {
        std::fs::remove_file(&self.0.images[0].file.path).unwrap();
    }
}

#[test]
fn markdown_images_are_embedded_in_every_rich_output() {
    let fixture = ImageFixture::new();
    for format in [DocumentFormat::Docx, DocumentFormat::Pptx] {
        let bytes = format.render(&fixture.0).unwrap();
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        let prefix = if matches!(format, DocumentFormat::Docx) {
            "word/media/"
        } else {
            "ppt/media/"
        };
        let image = archive
            .file_names()
            .find(|name| name.starts_with(prefix) && name.len() > prefix.len())
            .unwrap()
            .to_owned();
        let mut bytes = Vec::new();
        archive
            .by_name(&image)
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        assert_eq!(image::load_from_memory(&bytes).unwrap().width(), 32);
    }
    let html = String::from_utf8(DocumentFormat::Html.render(&fixture.0).unwrap()).unwrap();
    assert!(html.contains("data:image/png;base64,") && html.contains("示意图"));
    assert!(!html.contains(fixture.0.images[0].file.path.to_str().unwrap()));
    let pdf = DocumentFormat::Pdf.render(&fixture.0).unwrap();
    assert!(pdf.starts_with(b"%PDF-"));
}

#[test]
fn changed_image_bytes_fail_instead_of_publishing_an_unpinned_picture() {
    let fixture = ImageFixture::new();
    std::fs::write(&fixture.0.images[0].file.path, b"changed").unwrap();
    assert!(DocumentFormat::Docx
        .render(&fixture.0)
        .unwrap_err()
        .contains("changed"));
    let text = String::from_utf8(DocumentFormat::Text.render(&fixture.0).unwrap()).unwrap();
    assert!(text.contains("[Image: 示意图]") && !text.contains("figure.png"));
}

/// A conversion embeds bytes, so it can only embed pictures this Space holds. A reference to
/// anything else says what it shows and nothing more, which is what keeps a document full of
/// badges convertible.
#[test]
fn a_picture_the_space_does_not_hold_reads_as_its_alt_text() {
    let source = DocumentSource {
        markdown: "![build passing](https://img.example/badge.svg) ok\n\n![](gone.png)".into(),
        ..DocumentSource::default()
    };
    let text = String::from_utf8(DocumentFormat::Text.render(&source).unwrap()).unwrap();
    assert_eq!(text, "build passing ok\n\n");
    assert!(DocumentFormat::Docx.render(&source).is_ok());
}

#[test]
fn jpeg_camera_orientation_is_applied_before_embedding() {
    let mut jpeg = Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(2, 3)
        .write_to(&mut jpeg, image::ImageFormat::Jpeg)
        .unwrap();
    // Little-endian TIFF orientation 6: rotate the stored sensor pixels 90 degrees.
    let exif = b"Exif\0\0II\x2a\0\x08\0\0\0\x01\0\x12\x01\x03\0\x01\0\0\0\x06\0\0\0\0\0\0\0";
    let mut bytes = jpeg.into_inner();
    let mut segment = vec![0xff, 0xe1];
    segment.extend_from_slice(&u16::try_from(exif.len() + 2).unwrap().to_be_bytes());
    segment.extend_from_slice(exif);
    drop(bytes.splice(2..2, segment));
    let normalized = super::super::images::decode(&bytes).unwrap();
    assert_eq!((normalized.width, normalized.height), (3, 2));
    let embedded = image::load_from_memory(&normalized.bytes).unwrap();
    assert_eq!((embedded.width(), embedded.height()), (3, 2));
}

#[test]
fn repeated_references_share_the_embedded_copy_budget() {
    let images = Images(
        BTreeMap::from([(
            "figure".into(),
            Image {
                bytes: vec![0; 1024 * 1024],
                width: 1,
                height: 1,
            },
        )]),
        Cell::new(0),
    );
    for _ in 0..64 {
        assert_eq!(images.clone_bytes("figure").unwrap().len(), 1024 * 1024);
    }
    assert!(images.clone_bytes("figure").unwrap_err().contains("64 MiB"));
}
