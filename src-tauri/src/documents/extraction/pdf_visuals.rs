use std::collections::{BTreeMap, BTreeSet};

use pdf_inspector::types::ItemType;
use pdf_inspector::TextItem;

use super::visual::{is_meaningful_point_size, is_repeated_watermark_or_template};
use crate::documents::pdf_input::{PdfInput, PdfRecognitionReason, PdfRegion};

const MIN_TEXT_STRADDLE_PT: f64 = 8.0;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SizeKey(u32, u32);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct RectKey(i32, i32, u32, u32);

struct Repetition {
    pages_by_rect: BTreeMap<RectKey, usize>,
    pages_by_size: BTreeMap<SizeKey, usize>,
    max_per_page_by_size: BTreeMap<SizeKey, usize>,
}

impl Repetition {
    fn from_pages(pages: &BTreeMap<u32, Vec<PdfRegion>>) -> Self {
        let mut pages_by_rect = BTreeMap::<RectKey, usize>::new();
        let mut pages_by_size = BTreeMap::<SizeKey, usize>::new();
        let mut max_per_page_by_size = BTreeMap::<SizeKey, usize>::new();
        for regions in pages.values() {
            let mut page_rects = BTreeSet::new();
            let mut count_by_size = BTreeMap::<SizeKey, usize>::new();
            for rect in regions {
                page_rects.insert(rect_key(rect));
                *count_by_size.entry(size_key(rect)).or_default() += 1;
            }
            for key in page_rects {
                *pages_by_rect.entry(key).or_default() += 1;
            }
            for (key, count) in count_by_size {
                *pages_by_size.entry(key).or_default() += 1;
                let max = max_per_page_by_size.entry(key).or_default();
                *max = (*max).max(count);
            }
        }
        Self {
            pages_by_rect,
            pages_by_size,
            max_per_page_by_size,
        }
    }

    fn is_watermark(&self, rect: &PdfRegion) -> bool {
        let pages_with_rect = self
            .pages_by_rect
            .get(&rect_key(rect))
            .copied()
            .unwrap_or(0);
        let size = size_key(rect);
        let max_on_page = self.max_per_page_by_size.get(&size).copied().unwrap_or(0);
        let pages_with_size = self.pages_by_size.get(&size).copied().unwrap_or(0);
        is_repeated_watermark_or_template(pages_with_rect, 1)
            || is_repeated_watermark_or_template(pages_with_size, max_on_page) && max_on_page >= 3
    }
}

pub(crate) struct PdfVisual {
    pub input: PdfInput,
    pub context: String,
}

impl PdfVisual {
    /// The page this picture is on. Visuals are planned page by page, so a list of them is in
    /// page order and a boundary page splits it where a reader of the document would split it.
    pub(crate) fn page(&self) -> u32 {
        self.input.page()
    }
}

pub(crate) fn group_by_page(positions: Vec<TextItem>) -> BTreeMap<u32, Vec<TextItem>> {
    let mut items_by_page = BTreeMap::<u32, Vec<TextItem>>::new();
    for item in positions {
        items_by_page.entry(item.page).or_default().push(item);
    }
    items_by_page
}

pub(crate) fn extract_figure_regions(
    positioned_pages: &BTreeMap<u32, Vec<TextItem>>,
) -> Result<BTreeMap<u32, Vec<PdfRegion>>, String> {
    let mut regions_by_page = BTreeMap::new();
    for (&page, items) in positioned_pages {
        let regions = candidate_regions(items)?;
        if !regions.is_empty() {
            regions_by_page.insert(page, regions);
        }
    }
    let repetition = Repetition::from_pages(&regions_by_page);
    Ok(regions_by_page
        .into_iter()
        .filter_map(|(page, regions)| {
            let kept = regions
                .into_iter()
                .filter(|rect| !repetition.is_watermark(rect))
                .collect::<Vec<_>>();
            (!kept.is_empty()).then_some((page, kept))
        })
        .collect())
}

pub(super) fn plan(
    page: u32,
    reason: Option<&str>,
    items: &[TextItem],
    regions: Vec<PdfRegion>,
) -> Result<Vec<PdfVisual>, String> {
    if let Some(reason) = PdfRecognitionReason::from_parser(reason)? {
        return Ok(vec![PdfVisual {
            input: PdfInput::Page { page, reason },
            context: String::new(),
        }]);
    }
    Ok(regions
        .into_iter()
        .map(|rect| PdfVisual {
            input: PdfInput::Image { page, rect },
            context: context(items, &rect),
        })
        .collect())
}

fn candidate_regions(items: &[TextItem]) -> Result<Vec<PdfRegion>, String> {
    let mut regions = items
        .iter()
        .filter(|item| matches!(item.item_type, ItemType::Image))
        .map(region)
        .collect::<Result<Vec<_>, _>>()?;
    regions.retain(|rect| is_meaningful_size(rect) && !straddles_body_text(rect, items));
    regions.sort_by(|a, b| {
        b.y.total_cmp(&a.y)
            .then(a.x.total_cmp(&b.x))
            .then(a.width.total_cmp(&b.width))
            .then(a.height.total_cmp(&b.height))
    });
    regions.dedup();
    Ok(regions)
}

fn region(item: &TextItem) -> Result<PdfRegion, String> {
    let rect = PdfRegion {
        x: f64::from(item.x.min(item.x + item.width)),
        y: f64::from(item.y.min(item.y + item.height)),
        width: f64::from(item.width.abs()),
        height: f64::from(item.height.abs()),
    };
    rect.validate()?;
    Ok(rect)
}

fn is_meaningful_size(rect: &PdfRegion) -> bool {
    is_meaningful_point_size(rect.width, rect.height)
}

fn size_key(rect: &PdfRegion) -> SizeKey {
    SizeKey(rect.width.round() as u32, rect.height.round() as u32)
}

fn rect_key(rect: &PdfRegion) -> RectKey {
    RectKey(
        rect.x.round() as i32,
        rect.y.round() as i32,
        rect.width.round() as u32,
        rect.height.round() as u32,
    )
}

fn straddles_body_text(rect: &PdfRegion, items: &[TextItem]) -> bool {
    items
        .iter()
        .filter(|item| matches!(item.item_type, ItemType::Text) && !item.text.trim().is_empty())
        .filter(|item| item_straddles_rect(item, rect))
        .take(2)
        .count()
        >= 2
}

fn item_straddles_rect(item: &TextItem, rect: &PdfRegion) -> bool {
    let left = f64::from(item.x.min(item.x + item.width));
    let right = left + f64::from(item.width.abs());
    let bottom = f64::from(item.y.min(item.y + item.height));
    let top = bottom + f64::from(item.height.abs());
    let overlaps_y = top > rect.y && bottom < rect.y + rect.height;
    let inside_width = (right.min(rect.x + rect.width) - left.max(rect.x)).max(0.0);
    let outside_width = (right - left) - inside_width;
    overlaps_y && inside_width >= MIN_TEXT_STRADDLE_PT && outside_width >= MIN_TEXT_STRADDLE_PT
}

fn context(items: &[TextItem], rect: &PdfRegion) -> String {
    let mut nearby = items
        .iter()
        .filter(|item| matches!(item.item_type, ItemType::Text) && !item.text.trim().is_empty())
        .collect::<Vec<_>>();
    if nearby.len() > 12 {
        nearby.select_nth_unstable_by(12, |a, b| distance(a, rect).total_cmp(&distance(b, rect)));
    }
    nearby.truncate(12);
    nearby.sort_by(|a, b| b.y.total_cmp(&a.y).then(a.x.total_cmp(&b.x)));
    let text = nearby
        .iter()
        .map(|item| item.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    super::text::bounded_utf16_text(&text, 4000)
}

fn distance(item: &TextItem, rect: &PdfRegion) -> f64 {
    let x = f64::from(item.x + item.width / 2.0);
    let y = f64::from(item.y + item.height / 2.0);
    let dx = (rect.x - x).max(0.0).max(x - rect.x - rect.width);
    let dy = (rect.y - y).max(0.0).max(y - rect.y - rect.height);
    dx * dx + dy * dy
}

#[cfg(test)]
#[path = "tests/pdf_visuals.rs"]
mod tests;
