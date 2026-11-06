use super::{encode, frame, EDGE, MARGIN};
use image::{Rgba, RgbaImage};

/// A lifted sheet holding one red sticker twice as wide as it is tall, with transparent pixels
/// around it whose colour channels are black.
fn sheet() -> RgbaImage {
    RgbaImage::from_fn(300, 200, |x, y| {
        if (50..250).contains(&x) && (50..150).contains(&y) {
            Rgba([230, 20, 20, 255])
        } else {
            Rgba([0, 0, 0, 0])
        }
    })
}

/// Inclusive bounds of the covered pixels.
fn covered(image: &RgbaImage) -> [u32; 4] {
    image
        .enumerate_pixels()
        .filter(|(_, _, pixel)| pixel[3] > 0)
        .fold([u32::MAX, u32::MAX, 0, 0], |[l, t, r, b], (x, y, _)| {
            [l.min(x), t.min(y), r.max(x), b.max(y)]
        })
}

#[test]
fn a_sticker_is_a_square_main_image_that_never_touches_its_frame() {
    let framed = frame(&sheet(), [50, 50, 249, 149]);
    assert_eq!(framed.dimensions(), (EDGE, EDGE));
    let [left, top, right, bottom] = covered(&framed);
    assert!(
        left >= MARGIN && top >= MARGIN,
        "clear of the top left: {left}, {top}"
    );
    assert!(right < EDGE - MARGIN && bottom < EDGE - MARGIN);
    assert_eq!(
        right - left + 1,
        EDGE - 2 * MARGIN,
        "the long side fills the clear space"
    );
    let height = bottom - top + 1;
    assert!(
        height.abs_diff((EDGE - 2 * MARGIN) / 2) <= 2,
        "the aspect is kept: {height}"
    );
    assert_eq!(
        top.abs_diff(EDGE - 1 - bottom),
        0,
        "centred: {top} and {bottom}"
    );
}

#[test]
fn a_small_sticker_is_enlarged_to_the_frame() {
    let framed = frame(&sheet(), [50, 50, 69, 69]);
    let [left, _, right, _] = covered(&framed);
    assert_eq!(right - left + 1, EDGE - 2 * MARGIN);
}

#[test]
fn the_transparent_ground_does_not_darken_the_edge() {
    // The crop includes transparent black around the sticker, which unweighted resampling would
    // mix into the red edge.
    let framed = frame(&sheet(), [40, 40, 259, 159]);
    for pixel in framed.pixels().filter(|pixel| pixel[3] > 0) {
        assert!(pixel[0] > 200, "edge stays red: {pixel:?}");
    }
}

#[test]
fn a_sticker_encodes_as_png() {
    let bytes = encode(&frame(&sheet(), [50, 50, 249, 149])).expect("png");
    assert_eq!(&bytes[1..4], b"PNG");
}
