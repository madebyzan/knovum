use super::lift;
use image::{Rgba, RgbaImage};

const WHITE: Rgba<u8> = Rgba([255, 255, 255, 255]);
const INK: Rgba<u8> = Rgba([20, 20, 20, 255]);

/// A white sheet holding one sticker: a dark outline ring with a white belly inside it.
fn sheet() -> RgbaImage {
    let mut sheet = RgbaImage::from_pixel(60, 60, WHITE);
    for y in 10..50 {
        for x in 10..50 {
            if !(14..46).contains(&x) || !(14..46).contains(&y) {
                sheet.put_pixel(x, y, INK);
            }
        }
    }
    sheet
}

#[test]
fn the_backdrop_the_edge_reaches_is_lifted_and_an_enclosed_white_stays() {
    let lifted = lift(&sheet()).expect("sheet");
    assert_eq!(lifted.get_pixel(2, 2)[3], 0, "the backdrop is transparent");
    assert_eq!(
        *lifted.get_pixel(30, 30),
        WHITE,
        "the white belly is the sticker's"
    );
    assert_eq!(*lifted.get_pixel(12, 30), INK, "the outline is untouched");
}

#[test]
fn a_rim_blended_with_the_backdrop_keeps_only_the_sticker_share() {
    let mut source = sheet();
    // Antialiasing left a pale blend of ink and backdrop on the outline's outer edge.
    source.put_pixel(9, 30, Rgba([215, 215, 215, 255]));
    let rim = *lift(&source).expect("sheet").get_pixel(9, 30);
    assert!(rim[3] > 0 && rim[3] < 128, "partly covered: {rim:?}");
    assert!(
        rim[0] < 120,
        "its colour is the ink's, not the backdrop's: {rim:?}"
    );
}

#[test]
fn a_transparent_sheet_keeps_its_own_alpha() {
    let mut source = RgbaImage::from_pixel(40, 40, Rgba([0, 0, 0, 0]));
    source.put_pixel(20, 20, INK);
    source.put_pixel(21, 20, Rgba([200, 0, 0, 8]));
    let lifted = lift(&source).expect("sheet");
    assert_eq!(*lifted.get_pixel(20, 20), INK);
    assert_eq!(
        lifted.get_pixel(21, 20)[3],
        0,
        "a trace of alpha is backdrop"
    );
}

#[test]
fn a_photograph_is_not_a_sticker_sheet() {
    let photo = RgbaImage::from_fn(40, 40, |x, y| {
        Rgba([(x * 6) as u8, (y * 6) as u8, ((x + y) * 3) as u8, 255])
    });
    assert!(lift(&photo).is_err());
}
