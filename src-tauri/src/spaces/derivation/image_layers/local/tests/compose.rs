use super::encode;
use crate::spaces::derivation::image_layers::local::{Decomposition, Layers};
use image::{ImageFormat, Rgba, RgbaImage};
use tokio_util::sync::CancellationToken;

/// A gradient so no layer is accidentally uniform, which would make it contentless.
fn source() -> RgbaImage {
    RgbaImage::from_fn(4, 2, |x, _| Rgba([(x * 60) as u8, 10, 20, 255]))
}

fn layers(labels: Vec<u8>, count: u8, backdrop: Option<RgbaImage>) -> Layers {
    Layers {
        decomposition: Decomposition {
            labels,
            count,
            width: 4,
            height: 2,
        },
        alpha: None,
        backdrop,
    }
}

fn decode(png: &[u8]) -> RgbaImage {
    image::load_from_memory_with_format(png, ImageFormat::Png)
        .expect("layer png")
        .to_rgba8()
}

#[test]
fn a_subject_is_cropped_to_the_pixels_it_carries() {
    let labels = vec![0, 1, 1, 2, 0, 1, 2, 2];
    let encoded = encode(
        &source(),
        &layers(labels, 2, None),
        &CancellationToken::new(),
    )
    .expect("layers");
    assert_eq!(encoded.len(), 2, "one layer per subject, no backdrop");
    let first = decode(&encoded[0]);
    assert_eq!(
        first.dimensions(),
        (2, 2),
        "the first subject spans two columns and two rows"
    );
    assert_eq!(
        first.get_pixel(0, 0).0,
        [60, 10, 20, 255],
        "the crop starts at the subject, not at the canvas"
    );
    assert_eq!(
        first.get_pixel(1, 1)[3],
        0,
        "pixels of other subjects stay transparent inside the crop"
    );
    assert_eq!(decode(&encoded[1]).dimensions(), (2, 2));
}

#[test]
fn a_subject_that_carries_no_pixel_is_not_published() {
    let encoded = encode(
        &source(),
        &layers(vec![0, 1, 1, 0, 0, 1, 0, 0], 2, None),
        &CancellationToken::new(),
    )
    .expect("layers");
    assert_eq!(encoded.len(), 1, "the empty second label is left out");
}

#[test]
fn a_backdrop_is_published_only_when_it_carries_content() {
    let labels = vec![1, 1, 0, 0, 0, 0, 0, 0];
    let flat = RgbaImage::from_pixel(4, 2, Rgba([9, 9, 9, 255]));
    let encoded = encode(
        &source(),
        &layers(labels.clone(), 1, Some(flat)),
        &CancellationToken::new(),
    )
    .expect("layers");
    assert_eq!(
        encoded.len(),
        1,
        "a backdrop of one repeated pixel is not a layer"
    );
    let encoded = encode(
        &source(),
        &layers(labels, 1, Some(source())),
        &CancellationToken::new(),
    )
    .expect("layers");
    assert_eq!(encoded.len(), 2, "a backdrop with detail leads the stack");
    assert_eq!(decode(&encoded[0]).dimensions(), (4, 2));
}

#[test]
fn a_soft_alpha_replaces_the_binary_mask_of_the_layer_that_owns_it() {
    let mut layers = layers(vec![1, 1, 0, 0, 0, 0, 0, 0], 1, None);
    layers.alpha = Some((1, vec![255, 128, 0, 0, 0, 0, 0, 0]));
    let encoded = encode(&source(), &layers, &CancellationToken::new()).expect("layers");
    let subject = decode(&encoded[0]);
    assert_eq!(subject.dimensions(), (2, 1));
    assert_eq!(subject.get_pixel(0, 0)[3], 255);
    assert_eq!(subject.get_pixel(1, 0)[3], 128, "soft coverage is kept");
}

#[test]
fn a_decomposition_from_another_canvas_is_rejected() {
    let elsewhere = Layers {
        decomposition: Decomposition {
            labels: vec![0, 1],
            count: 1,
            width: 2,
            height: 1,
        },
        alpha: None,
        backdrop: None,
    };
    assert!(encode(&source(), &elsewhere, &CancellationToken::new()).is_err());
}
