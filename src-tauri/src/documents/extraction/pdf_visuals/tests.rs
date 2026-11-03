use pdf_inspector::types::ItemType;
use pdf_inspector::TextItem;

use super::{extract_figure_regions, group_by_page, plan};
use crate::documents::pdf_input::{PdfInput, PdfRegion};

#[test]
fn filters_tiny_icons_and_text_straddling_watermarks_while_keeping_real_figures() {
    let pages = group_by_page(vec![
        // Real figure (240 x 160 pt) with a caption below it and a label inside it
        image_item(1, 80.0, 500.0, 240.0, 160.0),
        text_item(1, 100.0, 540.0, 80.0, 12.0, "Internal node label"),
        text_item(1, 80.0, 480.0, 200.0, 12.0, "Figure 1: Architecture"),
        // Tiny inline icon (16 x 16 pt)
        image_item(1, 72.0, 400.0, 16.0, 16.0),
        // Thin decorative rule (400 x 4 pt)
        image_item(1, 72.0, 380.0, 400.0, 4.0),
        // Watermark box stamped across body text lines (straddles left/right edge of text)
        image_item(1, 180.0, 200.0, 100.0, 80.0),
        text_item(
            1,
            72.0,
            250.0,
            360.0,
            12.0,
            "def __init__(self, args: ModelArgs, index: int):",
        ),
        text_item(
            1,
            72.0,
            230.0,
            360.0,
            12.0,
            "    assert args.n_embd % args.n_head == 0",
        ),
    ]);

    let regions = extract_figure_regions(&pages).unwrap();
    let page_items = pages.get(&1).unwrap();
    let visuals = plan(
        1,
        None,
        page_items,
        regions.get(&1).cloned().unwrap_or_default(),
    )
    .unwrap();

    assert_eq!(visuals.len(), 1);
    assert_eq!(
        visuals[0].input,
        PdfInput::Image {
            page: 1,
            rect: PdfRegion {
                x: 80.0,
                y: 500.0,
                width: 240.0,
                height: 160.0,
            },
        }
    );
}

#[test]
fn filters_tiled_watermarks_and_cross_page_stationary_furniture() {
    let mut items = Vec::new();
    // 4 pages with a 3-tile watermark grid per page (even on blank margins without text overlap)
    // and a stationary header logo at (480, 760, 64, 36) on every page.
    for page in 1..=4 {
        items.push(image_item(page, 480.0, 760.0, 64.0, 36.0));
        items.push(image_item(page, 80.0, 600.0, 96.0, 72.0));
        items.push(image_item(page, 260.0, 400.0, 96.0, 72.0));
        items.push(image_item(page, 420.0, 200.0, 96.0, 72.0));
    }
    // Real diagrams on page 2 and page 3
    items.push(image_item(2, 90.0, 300.0, 320.0, 180.0));
    items.push(image_item(3, 100.0, 240.0, 280.0, 200.0));

    let pages = group_by_page(items);
    let regions = extract_figure_regions(&pages).unwrap();

    assert_eq!(regions.keys().copied().collect::<Vec<_>>(), vec![2, 3]);
    assert_eq!(regions[&2].len(), 1);
    assert_eq!(regions[&2][0].width, 320.0);
    assert_eq!(regions[&3].len(), 1);
    assert_eq!(regions[&3][0].width, 280.0);
}

fn image_item(page: u32, x: f32, y: f32, width: f32, height: f32) -> TextItem {
    item(page, x, y, width, height, ItemType::Image, "")
}

fn text_item(page: u32, x: f32, y: f32, width: f32, height: f32, text: &str) -> TextItem {
    item(page, x, y, width, height, ItemType::Text, text)
}

fn item(
    page: u32,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    item_type: ItemType,
    text: &str,
) -> TextItem {
    TextItem {
        text: text.to_owned(),
        x,
        y,
        width,
        height,
        rotation: 0.0,
        advance_known: true,
        font: String::new(),
        font_tag: String::new(),
        legacy_symbol_rewrite: false,
        font_size: 12.0,
        page,
        is_bold: false,
        is_italic: false,
        font_weight: None,
        bold_source: None,
        fixed_pitch: None,
        fill_color: None,
        stroke_color: None,
        render_mode: None,
        is_underline: false,
        is_strikeout: false,
        item_type,
        mcid: None,
        baseline_shift: 0.0,
    }
}
