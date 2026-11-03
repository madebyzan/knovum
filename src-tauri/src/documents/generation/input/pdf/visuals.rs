use office_oxide::ir::{Element, Image, ImageFormat};
use std::collections::BTreeMap;

use crate::documents::{
    extraction::pdf_visuals::{extract_figure_regions, group_by_page},
    pdf_input::{PdfInput, PdfRecognitionReason, PdfRegion},
    rendering::PdfRenderer,
};

pub(super) struct Visuals {
    renderer: PdfRenderer,
    regions_by_page: BTreeMap<u32, Vec<PdfRegion>>,
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
        let positioned_pages = group_by_page(positions);
        let regions_by_page = extract_figure_regions(&positioned_pages)?;
        if regions_by_page.values().map(Vec::len).sum::<usize>() > 2000 {
            return Err("PDF exceeds 2000 image occurrences".into());
        }
        Ok(Self {
            renderer: PdfRenderer::from_bytes(bytes)?,
            regions_by_page,
            byte_size: 0,
            count: 0,
        })
    }

    pub fn page(&mut self, page: u32, scanned: bool) -> Result<Vec<Element>, String> {
        let regions = self.regions_by_page.remove(&page).unwrap_or_default();
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
            let raster = self.renderer.render(&input)?;
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
