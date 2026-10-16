use super::detect;
use image::{Rgba, RgbaImage};

const SHEET: u32 = 300;
const STICKER: u32 = 60;
const BORDER: u32 = 8;

/// Nine stickers on a uniform backdrop, three per row. Each sticker is a hollow frame holding a
/// detached inner block, which is the shape that must stay one layer rather than two.
fn sticker_sheet(backdrop: Rgba<u8>) -> RgbaImage {
    let mut sheet = RgbaImage::from_pixel(SHEET, SHEET, backdrop);
    for row in 0..3_u32 {
        for column in 0..3_u32 {
            let (left, top) = (column * 100 + 20, row * 100 + 20);
            for y in top..top + STICKER {
                for x in left..left + STICKER {
                    let framed = x < left + BORDER
                        || y < top + BORDER
                        || x >= left + STICKER - BORDER
                        || y >= top + STICKER - BORDER;
                    let inner =
                        (left + 23..left + 37).contains(&x) && (top + 23..top + 37).contains(&y);
                    if framed {
                        sheet.put_pixel(x, y, Rgba([200, 40, 40, 255]));
                    } else if inner {
                        sheet.put_pixel(x, y, Rgba([250, 250, 40, 255]));
                    }
                }
            }
        }
    }
    sheet
}

fn label_at(layers: &super::Layers, x: u32, y: u32) -> u8 {
    layers.decomposition.labels[(y * SHEET + x) as usize]
}

#[test]
fn a_sticker_sheet_splits_into_one_layer_per_sticker_in_reading_order() {
    let sheet = detect(&sticker_sheet(Rgba([255, 255, 255, 255])), 24).expect("sheet");
    assert_eq!(
        sheet.decomposition.count, 9,
        "every sticker is its own layer"
    );
    assert_eq!(label_at(&sheet, 22, 50), 1, "top left sticker is first");
    assert_eq!(label_at(&sheet, 122, 50), 2);
    assert_eq!(label_at(&sheet, 222, 50), 3);
    assert_eq!(label_at(&sheet, 22, 150), 4, "the second row follows");
    assert_eq!(label_at(&sheet, 222, 250), 9, "bottom right is last");
    assert_eq!(label_at(&sheet, 5, 5), 0, "the backdrop stays unassigned");
    assert_eq!(
        label_at(&sheet, 50, 50),
        1,
        "the detached inner block joins the sticker that contains it"
    );
}

#[test]
fn a_transparent_sheet_is_detected_through_its_alpha_channel() {
    let sheet = detect(&sticker_sheet(Rgba([0, 0, 0, 0])), 24).expect("sheet");
    assert_eq!(sheet.decomposition.count, 9);
    assert_eq!(label_at(&sheet, 5, 5), 0);
}

#[test]
fn the_layer_budget_caps_the_discovered_subjects() {
    let sheet = detect(&sticker_sheet(Rgba([255, 255, 255, 255])), 4).expect("sheet");
    assert_eq!(sheet.decomposition.count, 4);
    assert_eq!(
        sheet
            .decomposition
            .labels
            .iter()
            .filter(|label| **label > 4)
            .count(),
        0,
        "no label exceeds the budget"
    );
}

#[test]
fn sources_without_a_uniform_backdrop_are_not_flat_sheets() {
    let noisy = RgbaImage::from_fn(120, 120, |x, y| {
        Rgba([
            (x * 2 % 256) as u8,
            (y * 3 % 256) as u8,
            ((x + y) % 256) as u8,
            255,
        ])
    });
    assert!(detect(&noisy, 24).is_none());
    assert!(
        detect(
            &RgbaImage::from_pixel(120, 120, Rgba([255, 255, 255, 255])),
            24
        )
        .is_none(),
        "a backdrop with no subject carries no layers"
    );
}
