use super::*;

#[test]
fn local_mask_accepts_small_images_and_rejects_oversized_inputs() {
    let stroke = MaskStroke {
        diameter: 0.1,
        points: vec![0.5, 0.5, 0.5, 0.5],
    };
    let token = CancellationToken::new();
    assert!(rasterize(128, 96, std::slice::from_ref(&stroke), &token).is_ok());
    assert!(rasterize(8192, 8192, &[stroke], &token).is_err());
}

#[test]
fn mask_marks_only_selected_pixels() {
    let stroke = MaskStroke {
        diameter: 0.05,
        points: vec![0.5, 0.5, 0.5, 0.5],
    };
    let mask = rasterize(512, 512, &[stroke], &CancellationToken::new()).unwrap();
    assert_eq!(mask.get_pixel(0, 0)[0], 0);
    assert_eq!(mask.get_pixel(256, 256)[0], 255);
}

#[test]
fn cancelled_mask_does_not_continue_rasterizing() {
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let stroke = MaskStroke {
        diameter: 0.05,
        points: vec![0.0, 0.0, 1.0, 1.0],
    };
    assert!(rasterize(512, 512, &[stroke], &cancellation)
        .unwrap_err()
        .contains("cancelled"));
}

#[test]
fn diagonal_stroke_marks_its_corridor_only() {
    let stroke = MaskStroke {
        diameter: 0.01,
        points: vec![0.0, 0.0, 1.0, 1.0],
    };
    let mask = rasterize(512, 512, &[stroke], &CancellationToken::new()).unwrap();
    assert_eq!(mask.get_pixel(256, 256)[0], 255);
    assert_eq!(mask.get_pixel(256, 0)[0], 0);
}
