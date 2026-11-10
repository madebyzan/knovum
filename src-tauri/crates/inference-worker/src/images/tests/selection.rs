use super::{preprocess_shape, resize_logits, sam1_prompt, sam2_coords, MASK_GRID};

#[test]
fn longest_edge_resize_rounds_like_the_reference_implementation() {
    assert_eq!(preprocess_shape(3000, 4000), (768, 1024));
    assert_eq!(preprocess_shape(4000, 3000), (1024, 768));
    assert_eq!(preprocess_shape(1080, 1920), (576, 1024));
    assert_eq!(preprocess_shape(512, 512), (1024, 1024));
    // 599 * 1.024 = 613.376 truncates after the half-pixel offset.
    assert_eq!(preprocess_shape(599, 1000), (613, 1024));
}

#[test]
fn sam1_prompts_scale_into_the_resized_frame_and_append_the_padding_point() {
    let (coords, labels) = sam1_prompt(vec![100.0, 50.0], vec![1.0], (1000, 2000));
    assert_eq!(coords, [51.2, 25.6, 0.0, 0.0]);
    assert_eq!(labels, [1.0, -1.0]);
}

#[test]
fn sam2_prompts_scale_into_the_stretched_square() {
    assert_eq!(sam2_coords(vec![100.0, 100.0], (800, 1600)), [64.0, 128.0]);
}

#[test]
fn bilinear_logit_resampling_preserves_constant_fields() {
    let grid = MASK_GRID as u32;
    let resized = resize_logits(&[3.0; 4], (2, 2), (grid, grid));
    assert_eq!(resized.len() as u32, grid * grid);
    // Bilinear weights sum to one, so a constant field survives up to float rounding.
    assert!(resized.iter().all(|value| (value - 3.0).abs() < 1e-4));
}
