use std::io::Cursor;

use office_oxide::{
    ir::{DocumentIR, Element, HeaderFooter, Image, ImageFormat, Section},
    DocumentFormat,
};
use tokio_util::sync::CancellationToken;

use super::super::visuals;

fn png_bytes(width: u32, height: u32, fill: u8) -> Vec<u8> {
    let mut image = image::RgbImage::new(width, height);
    for pixel in image.pixels_mut() {
        *pixel = image::Rgb([fill, fill, fill]);
    }
    let mut bytes = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgb8(image)
        .write_to(&mut bytes, image::ImageFormat::Png)
        .unwrap();
    bytes.into_inner()
}

fn ir_image(bytes: Vec<u8>, decorative: bool, display_emu: Option<(u64, u64)>) -> Element {
    Element::Image(Image {
        data: Some(bytes),
        format: Some(ImageFormat::Png),
        decorative,
        display_width_emu: display_emu.map(|(w, _)| w),
        display_height_emu: display_emu.map(|(_, h)| h),
        ..Default::default()
    })
}

#[test]
fn skips_word_headers_footers_decorative_marks_and_tiny_icons() {
    let figure = png_bytes(160, 120, 40);
    let header_logo = png_bytes(120, 60, 80);
    let tiny_icon = png_bytes(16, 16, 120);
    let ir = DocumentIR {
        sections: vec![Section {
            header: Some(HeaderFooter {
                content: vec![ir_image(header_logo.clone(), false, None)],
            }),
            footer: Some(HeaderFooter {
                content: vec![ir_image(header_logo, false, None)],
            }),
            elements: vec![
                ir_image(tiny_icon, false, None),
                ir_image(figure.clone(), true, None),
                ir_image(figure.clone(), false, Some((100_000, 100_000))),
                ir_image(figure.clone(), false, None),
            ],
            ..Default::default()
        }],
        ..Default::default()
    };

    let collected = visuals::collect(&ir, DocumentFormat::Docx, &CancellationToken::new())
        .unwrap()
        .into_entries();

    assert_eq!(collected.len(), 1);
    assert_eq!(collected[0].data.as_deref(), Some(figure.as_slice()));
}

#[test]
fn filters_slide_master_logos_repeated_across_slides_while_keeping_unique_figures() {
    let slide_logo = png_bytes(96, 64, 10);
    let chart_one = png_bytes(240, 160, 100);
    let chart_two = png_bytes(240, 160, 200);
    let ir = DocumentIR {
        sections: vec![
            Section {
                elements: vec![
                    ir_image(slide_logo.clone(), false, None),
                    ir_image(chart_one.clone(), false, None),
                ],
                ..Default::default()
            },
            Section {
                elements: vec![
                    ir_image(slide_logo.clone(), false, None),
                    ir_image(chart_two.clone(), false, None),
                ],
                ..Default::default()
            },
            Section {
                elements: vec![ir_image(slide_logo, false, None)],
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let collected = visuals::collect(&ir, DocumentFormat::Pptx, &CancellationToken::new())
        .unwrap()
        .into_entries();

    assert_eq!(collected.len(), 2);
    assert_eq!(collected[0].data.as_deref(), Some(chart_one.as_slice()));
    assert_eq!(collected[1].data.as_deref(), Some(chart_two.as_slice()));
}
