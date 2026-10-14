use office_oxide::ir::{Element, Image, ImageFormat};
use pdf_inspector::types::ItemType;
use std::collections::BTreeMap;

use crate::documents::pdf_input::{PdfInput, PdfRecognitionReason, PdfRegion};

pub(super) struct Visuals {
    bytes: Vec<u8>,
    pages: BTreeMap<u32, Vec<PdfRegion>>,
    byte_size: usize,
    count: usize,
}

impl Visuals {
    pub fn new(bytes: &[u8]) -> Result<Self, String> {
        let positions = pdf_inspector::extract_text_with_positions_mem_in_frame(
            bytes,
            None,
            pdf_inspector::PositionFrame::Display,
        )
        .map_err(|error| format!("PDF image discovery failed: {error}"))?;
        let mut pages = BTreeMap::<u32, Vec<PdfRegion>>::new();
        for item in positions {
            if !matches!(item.item_type, ItemType::Image) {
                continue;
            }
            let region = PdfRegion {
                x: f64::from(item.x.min(item.x + item.width)),
                y: f64::from(item.y.min(item.y + item.height)),
                width: f64::from(item.width.abs()),
                height: f64::from(item.height.abs()),
            };
            region.validate()?;
            let page = pages.entry(item.page).or_default();
            if !page.contains(&region) {
                page.push(region);
            }
        }
        if pages.values().map(Vec::len).sum::<usize>() > 2000 {
            return Err("PDF exceeds 2000 image occurrences".into());
        }
        Ok(Self {
            bytes: bytes.to_vec(),
            pages,
            byte_size: 0,
            count: 0,
        })
    }

    pub fn page(&mut self, page: u32, scanned: bool) -> Result<Vec<Element>, String> {
        let mut regions = self.pages.remove(&page).unwrap_or_default();
        regions.sort_by(|a, b| b.y.total_cmp(&a.y).then(a.x.total_cmp(&b.x)));
        let inputs = if scanned {
            vec![PdfInput::Page {
                page,
                reason: PdfRecognitionReason::Scanned,
            }]
        } else {
            regions
                .into_iter()
                .map(|rect| PdfInput::Image { page, rect })
                .collect()
        };
        let mut elements = Vec::new();
        for input in inputs {
            self.count += 1;
            if self.count > 2000 {
                return Err("PDF exceeds 2000 image occurrences".into());
            }
            let raster = crate::documents::rendering::render_bytes(&self.bytes, &input)?;
            self.byte_size += raster.bytes.len();
            if self.byte_size > 64 * 1024 * 1024 {
                return Err("PDF images exceed 64 MiB".into());
            }
            elements.push(Element::Image(Image {
                data: Some(raster.bytes),
                format: Some(ImageFormat::Png),
                pixel_width: Some(raster.width),
                pixel_height: Some(raster.height),
                alt_text: Some(format!("Source page {page}")),
                ..Default::default()
            }));
        }
        Ok(elements)
    }
}
