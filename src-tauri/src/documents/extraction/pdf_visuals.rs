use pdf_inspector::types::ItemType;
use pdf_inspector::TextItem;

use crate::documents::pdf_input::{PdfInput, PdfRecognitionReason, PdfRegion};

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

pub(super) fn plan(
    page: u32,
    reason: Option<&str>,
    items: &[TextItem],
) -> Result<Vec<PdfVisual>, String> {
    let recognition = PdfRecognitionReason::from_parser(reason)?;
    if let Some(reason) = recognition {
        return Ok(vec![PdfVisual {
            input: PdfInput::Page { page, reason },
            context: String::new(),
        }]);
    }
    let mut regions = items
        .iter()
        .filter(|item| matches!(item.item_type, ItemType::Image))
        .map(region)
        .collect::<Result<Vec<_>, _>>()?;
    regions.sort_by(|a, b| {
        b.y.total_cmp(&a.y)
            .then(a.x.total_cmp(&b.x))
            .then(a.width.total_cmp(&b.width))
            .then(a.height.total_cmp(&b.height))
    });
    regions.dedup();
    Ok(regions
        .into_iter()
        .map(|rect| PdfVisual {
            input: PdfInput::Image { page, rect },
            context: context(items, &rect),
        })
        .collect())
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
