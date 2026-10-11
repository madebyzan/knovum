use crate::images::{
    depth::normalize,
    inpainting::{composite, known_mask},
    upscaling::quantize,
};

#[test]
fn upscale_quantization_matches_numpy_ties_to_even_and_clipping() {
    assert_eq!(quantize(0.5 / 255.0).unwrap(), 0);
    assert_eq!(quantize(1.5 / 255.0).unwrap(), 2);
    assert_eq!(quantize(-1.0).unwrap(), 0);
    assert_eq!(quantize(2.0).unwrap(), 255);
    assert!(quantize(f32::NAN).is_err());
}

#[test]
fn inpainting_inverts_supplier_polarity_and_preserves_unselected_channels() {
    let mask = [0, 255];
    assert_eq!(known_mask(&mask).unwrap(), [255, 0]);
    assert!(known_mask(&[0, 0]).is_err());
    assert!(known_mask(&[0, 127]).is_err());
    assert_eq!(
        composite(&[1, 2, 3, 4, 5, 6], &[7, 8, 9, 10, 11, 12], &mask).unwrap(),
        [1, 8, 3, 10, 5, 12]
    );
}

#[test]
fn depth_rejects_invalid_fields_and_normalizes_relative_extremes() {
    assert_eq!(normalize(&[2.0, 4.0, 6.0]).unwrap(), [0.0, 0.5, 1.0]);
    assert!(normalize(&[1.0, 1.0]).is_err());
    assert!(normalize(&[1.0, f32::INFINITY]).is_err());
}
