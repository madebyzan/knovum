
use super::*;

#[test]
fn mask_marks_only_selected_pixels() {
    let stroke = MaskStroke {
        diameter: 0.05,
        points: vec![0.5, 0.5, 0.5, 0.5],
    };
    let uri = encode(512, 512, &[stroke], &CancellationToken::new()).unwrap();
    let bytes = STANDARD
        .decode(uri.strip_prefix("data:image/png;base64,").unwrap())
        .unwrap();
    let mask = image::load_from_memory(&bytes).unwrap().to_luma8();
    assert_eq!(mask.get_pixel(0, 0)[0], 0);
    assert_eq!(mask.get_pixel(256, 256)[0], 255);
}

#[test]
fn openai_mask_makes_only_selected_pixels_transparent() {
    let stroke = MaskStroke {
        diameter: 0.05,
        points: vec![0.5, 0.5, 0.5, 0.5],
    };
    let bytes = encode_openai(512, 512, &[stroke], &CancellationToken::new()).unwrap();
    let mask = image::load_from_memory(&bytes).unwrap().to_rgba8();
    assert_eq!(mask.get_pixel(0, 0)[3], 255);
    assert_eq!(mask.get_pixel(256, 256)[3], 0);
}

#[test]
fn cancelled_mask_does_not_continue_rasterizing() {
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let stroke = MaskStroke {
        diameter: 0.05,
        points: vec![0.0, 0.0, 1.0, 1.0],
    };
    assert!(encode(512, 512, &[stroke], &cancellation)
        .unwrap_err()
        .contains("cancelled"));
}

#[test]
fn diagonal_stroke_marks_its_corridor_only() {
    let stroke = MaskStroke {
        diameter: 0.01,
        points: vec![0.0, 0.0, 1.0, 1.0],
    };
    let uri = encode(512, 512, &[stroke], &CancellationToken::new()).unwrap();
    let bytes = STANDARD
        .decode(uri.strip_prefix("data:image/png;base64,").unwrap())
        .unwrap();
    let mask = image::load_from_memory(&bytes).unwrap().to_luma8();
    assert_eq!(mask.get_pixel(256, 256)[0], 255);
    assert_eq!(mask.get_pixel(256, 0)[0], 0);
}
