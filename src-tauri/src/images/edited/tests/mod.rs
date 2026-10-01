use super::validate_source_dimensions;

#[test]
fn source_limits_reject_empty_and_oversized_images() {
    assert!(validate_source_dimensions((1, 1)).is_ok());
    for size in [(0, 1), (1, 0), (10_001, 1), (1, 10_001)] {
        assert!(validate_source_dimensions(size).is_err());
    }
    assert!(validate_source_dimensions((10_000, 10_000)).is_ok());
}

#[test]
fn native_export_matches_shared_preview_tone_fixtures() {
    #[derive(serde::Deserialize)]
    struct Fixture {
        name: String,
        width: u32,
        height: u32,
        source: Vec<u8>,
        adjustments: super::contract::Adjustments,
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
