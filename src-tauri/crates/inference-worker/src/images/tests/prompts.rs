use super::{build, foreground_point};
use knovum_inference_contracts::image_compute::{ImageSelectionPoint, ImageSelectionRect};

fn point(x: f32, y: f32, foreground: bool) -> ImageSelectionPoint {
    ImageSelectionPoint { x, y, foreground }
}

#[test]
fn prompt_uses_pixel_coordinates_and_distinct_box_labels() {
    let points = [point(0.5, 0.25, true)];
    let rect = ImageSelectionRect {
        x: 0.0,
        y: 0.0,
        width: 1.0,
        height: 1.0,
    };
    let (coords, labels) = build(&points, Some(&rect), 100, 200).unwrap();
    assert_eq!(coords, [100.0, 25.0, 0.0, 0.0, 200.0, 100.0]);
    assert_eq!(labels, [1.0, 2.0, 3.0]);
    assert!(build(&[point(f32::NAN, 0.5, true)], None, 100, 200).is_err());
    assert!(build(&[point(0.5, 0.5, false)], None, 100, 200).is_err());
}

#[test]
fn discovery_seeds_are_single_foreground_prompts_in_pixel_coordinates() {
    let (coords, labels) = foreground_point(&point(0.25, 0.5, true), 100, 200).unwrap();
    assert_eq!(coords, [50.0, 50.0]);
    assert_eq!(labels, [1.0]);
    assert!(foreground_point(&point(0.25, 0.5, false), 100, 200).is_err());
    assert!(foreground_point(&point(1.5, 0.5, true), 100, 200).is_err());
}
