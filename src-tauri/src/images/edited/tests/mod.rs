use super::{validate_source_dimensions, Encoding};

#[test]
fn source_limits_reject_empty_and_oversized_images() {
    assert!(validate_source_dimensions((1, 1)).is_ok());
    for size in [(0, 1), (1, 0), (10_001, 1), (1, 10_001)] {
        assert!(validate_source_dimensions(size).is_err());
    }
    assert!(validate_source_dimensions((10_000, 10_000)).is_ok());
}

#[test]
fn rust_export_matches_shared_preview_tone_fixtures() {
    #[derive(serde::Deserialize)]
    struct Fixture {
        name: String,
        width: u32,
        height: u32,
        source: Vec<u8>,
        adjustments: super::contracts::Adjustments,
        expected: Vec<u8>,
    }
    let fixtures: Vec<Fixture> = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../tests/fixtures/image-tone.json"
    )))
    .unwrap();
    for fixture in fixtures {
        let mut image =
            image::RgbaImage::from_raw(fixture.width, fixture.height, fixture.source).unwrap();
        let tone = super::tone::ToneAdjuster::new(&fixture.adjustments);
        for (x, y, pixel) in image.enumerate_pixels_mut() {
            tone.apply(pixel, x, y, fixture.width, fixture.height);
        }
        assert_eq!(image.into_raw(), fixture.expected, "{}", fixture.name);
    }
}

/// Every encoding writes the format it names, so a destination that asks for one never receives
/// another format's bytes - and a toolchain that cannot write one fails here rather than in an
/// export.
#[test]
fn every_encoding_writes_the_format_it_names() {
    let path = std::env::temp_dir().join(format!("knovum-encoding-{}.png", uuid::Uuid::new_v4()));
    let mut source = std::io::Cursor::new(Vec::new());
    image::DynamicImage::new_rgba8(4, 3)
        .write_to(&mut source, image::ImageFormat::Png)
        .unwrap();
    std::fs::write(&path, source.into_inner()).unwrap();
    for (encoding, format) in [
        (Encoding::Png, image::ImageFormat::Png),
        (Encoding::Jpeg, image::ImageFormat::Jpeg),
        (Encoding::WebP, image::ImageFormat::WebP),
        (Encoding::Tiff, image::ImageFormat::Tiff),
    ] {
        let bytes = super::render_source_as(&path, &serde_json::Value::Null, encoding).unwrap();
        assert_eq!(
            image::guess_format(&bytes).unwrap(),
            format,
            "{} names its format",
            encoding.extension()
        );
        assert_eq!(image::load_from_memory(&bytes).unwrap().width(), 4);
    }
    std::fs::remove_file(&path).unwrap();
}
