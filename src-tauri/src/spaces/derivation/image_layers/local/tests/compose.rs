use super::encode;
use crate::spaces::derivation::image_layers::local::{Decomposition, Layers};
use image::{ImageFormat, Rgba, RgbaImage};
use tokio_util::sync::CancellationToken;

fn source() -> RgbaImage {
    RgbaImage::from_fn(4, 2, |x, _| Rgba([(x * 60) as u8, 10, 20, 255]))
}

fn decode(png: &[u8]) -> RgbaImage {
    image::load_from_memory_with_format(png, ImageFormat::Png)
        .expect("layer png")
        .to_rgba8()
}

/// Layers are written in stack order, so compositing them back must reproduce the source.
fn flatten(layers: &[Vec<u8>], width: u32, height: u32) -> RgbaImage {
    let mut canvas = RgbaImage::new(width, height);
    for png in layers {
        image::imageops::overlay(&mut canvas, &decode(png), 0, 0);
    }
    canvas
}

#[test]
fn layers_keep_the_canvas_and_composite_back_into_the_source() {
    let layers = Layers {
        decomposition: Decomposition {
            labels: vec![0, 1, 1, 2, 0, 1, 2, 2],
            count: 2,
            width: 4,
            height: 2,
        },
        alpha: None,
        backdrop: None,
    };
    let source = source();
    let encoded = encode(&source, &layers, &CancellationToken::new()).expect("layers");
    assert_eq!(encoded.len(), 3, "a backdrop plus two subjects");
    for png in &encoded {
        let layer = decode(png);
        assert_eq!((layer.width(), layer.height()), (4, 2));
    }
    assert_eq!(flatten(&encoded, 4, 2), source);
    let backdrop = decode(&encoded[0]);
    assert_eq!(backdrop.get_pixel(0, 0)[3], 255, "unlabelled pixels remain");
    assert_eq!(
        backdrop.get_pixel(1, 0)[3],
        0,
        "the backdrop keeps a hole where a subject was lifted"
    );
    let first = decode(&encoded[1]);
    assert_eq!(first.get_pixel(1, 0)[3], 255);
    assert_eq!(first.get_pixel(3, 0)[3], 0, "other labels are transparent");
}

#[test]
fn a_soft_alpha_replaces_the_binary_mask_of_the_layer_that_owns_it() {
    let layers = Layers {
        decomposition: Decomposition {
            labels: vec![1, 1, 0, 0, 0, 0, 0, 0],
            count: 1,
            width: 4,
            height: 2,
        },
        alpha: Some((1, vec![255, 128, 0, 0, 0, 0, 0, 0])),
        backdrop: Some(RgbaImage::from_pixel(4, 2, Rgba([9, 9, 9, 255]))),
    };
    let encoded = encode(&source(), &layers, &CancellationToken::new()).expect("layers");
    let subject = decode(&encoded[1]);
    assert_eq!(subject.get_pixel(0, 0)[3], 255);
    assert_eq!(subject.get_pixel(1, 0)[3], 128, "soft coverage is kept");
    assert_eq!(decode(&encoded[0]).get_pixel(1, 0).0, [9, 9, 9, 255]);
}

#[test]
fn a_decomposition_from_another_canvas_is_rejected() {
    let layers = Layers {
        decomposition: Decomposition {
            labels: vec![0, 1],
            count: 1,
            width: 2,
            height: 1,
        },
        alpha: None,
        backdrop: None,
    };
    assert!(encode(&source(), &layers, &CancellationToken::new()).is_err());
}
