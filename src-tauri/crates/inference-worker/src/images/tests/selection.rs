use crate::images::selection::prompt;
use knovum_inference_contracts::image_compute::{ImageSelectionPoint, ImageSelectionRect};

#[test]
fn prompt_uses_pixel_coordinates_and_distinct_box_labels() {
    let points = [ImageSelectionPoint {
        x: 0.5,
        y: 0.25,
        foreground: true,
    }];
    let rect = ImageSelectionRect {
        x: 0.0,
        y: 0.0,
        width: 1.0,
        height: 1.0,
    };
    let (coords, labels) = prompt(&points, Some(&rect), 100, 200).unwrap();
    assert_eq!(coords, [100.0, 25.0, 0.0, 0.0, 200.0, 100.0]);
    assert_eq!(labels, [1.0, 2.0, 3.0]);
    assert!(prompt(
        &[ImageSelectionPoint {
            x: f32::NAN,
            y: 0.5,
            foreground: true
        }],
        None,
        100,
        200
    )
    .is_err());
    assert!(prompt(
        &[ImageSelectionPoint {
            x: 0.5,
            y: 0.5,
            foreground: false
        }],
        None,
        100,
        200
    )
    .is_err());
}
