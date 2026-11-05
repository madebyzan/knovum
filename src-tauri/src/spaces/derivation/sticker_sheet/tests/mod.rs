use super::{backdrop, sticker, stickers, SET};
use image::{Rgba, RgbaImage};

const PITCH: u32 = 120;

/// A white sheet of two rows of three stickers. Each sticker is an outlined character with a pale
/// belly and a caption of separate glyphs under it.
fn sheet() -> RgbaImage {
    let mut sheet = RgbaImage::from_pixel(3 * PITCH, 2 * PITCH, Rgba([255, 255, 255, 255]));
    for row in 0..2 {
        for column in 0..3 {
            let (left, top) = (column * PITCH + 30, row * PITCH + 15);
            for y in top..top + 60 {
                for x in left..left + 60 {
                    let outline =
                        !(left + 4..left + 56).contains(&x) || !(top + 4..top + 56).contains(&y);
                    let color = if outline {
                        [40, 30, 30]
                    } else {
                        [250, 248, 245]
                    };
                    sheet.put_pixel(x, y, Rgba([color[0], color[1], color[2], 255]));
                }
            }
            for glyph in 0..3 {
                for y in top + 68..top + 80 {
                    for x in left + 6 + glyph * 18..left + 18 + glyph * 18 {
                        sheet.put_pixel(x, y, Rgba([200, 40, 40, 255]));
                    }
                }
            }
        }
    }
    sheet
}

#[test]
fn a_sheet_slices_into_one_framed_sticker_per_cell() {
    let found = stickers(&backdrop::lift(&sheet()).expect("sheet"));
    assert_eq!(found.len(), 6);
    for framed in &found {
        assert_eq!(framed.dimensions(), (sticker::EDGE, sticker::EDGE));
        assert_eq!(
            framed.get_pixel(0, 0)[3],
            0,
            "the frame's ground is transparent"
        );
        let centre = framed.get_pixel(sticker::EDGE / 2, sticker::EDGE / 2 - 20);
        assert_eq!(
            centre[3], 255,
            "the pale belly is solid, not lifted with the backdrop"
        );
    }
    SET.validate_count(found.len()).expect("a sheet of six");
}

#[test]
fn a_single_sticker_is_not_a_sheet() {
    let mut single = RgbaImage::from_pixel(PITCH, PITCH, Rgba([255, 255, 255, 255]));
    for y in 30..90 {
        for x in 30..90 {
            single.put_pixel(x, y, Rgba([40, 30, 30, 255]));
        }
    }
    let found = stickers(&backdrop::lift(&single).expect("sheet"));
    assert_eq!(found.len(), 1);
    assert_eq!(
        SET.validate_count(found.len()).unwrap_err(),
        SET.unsupported
    );
}
