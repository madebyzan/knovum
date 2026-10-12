use objc2_core_foundation::{CFData, CGPoint, CGRect, CGSize, CFURL};
use objc2_core_graphics::{
    CGBitmapContextCreate, CGColorSpace, CGContext, CGDataProvider, CGImageAlphaInfo,
    CGImageByteOrderInfo, CGPDFBox, CGPDFDocument, CGPDFPage,
};
use std::{io::Cursor, path::Path};

pub(super) fn render(
    source: &Path,
    page: u32,
    region: Option<crate::documents::pdf_input::PdfRegion>,
) -> Result<super::PdfRaster, String> {
    let url = CFURL::from_file_path(source).ok_or("PDF path cannot be represented as a URL")?;
    let document = CGPDFDocument::with_url(Some(&url)).ok_or("Core Graphics could not open PDF")?;
    render_document(&document, page, region)
}

pub(super) fn render_bytes(
    bytes: &[u8],
    page: u32,
    region: Option<crate::documents::pdf_input::PdfRegion>,
) -> Result<super::PdfRaster, String> {
    let data = CFData::from_bytes(bytes);
    let provider = CGDataProvider::with_cf_data(Some(&data))
        .ok_or("Core Graphics could not create a PDF data provider")?;
    let document = CGPDFDocument::with_provider(Some(&provider))
        .ok_or("Core Graphics could not open PDF snapshot")?;
    render_document(&document, page, region)
}

fn render_document(
    document: &CGPDFDocument,
    page: u32,
    region: Option<crate::documents::pdf_input::PdfRegion>,
) -> Result<super::PdfRaster, String> {
    if !CGPDFDocument::is_unlocked(Some(document)) {
        return Err("PDF requires a password".into());
    }
    let page =
        CGPDFDocument::page(Some(document), page as usize).ok_or("PDF page does not exist")?;
    let (bounds, box_kind) = visible_bounds(&page);
    let rotation = CGPDFPage::rotation_angle(Some(&page)).rem_euclid(180);
    let (w, h) = if rotation == 90 {
        (bounds.size.height, bounds.size.width)
    } else {
        (bounds.size.width, bounds.size.height)
    };
    let crop = super::viewport(w, h, region)?;
    let (width, height) = super::dimensions(crop.width, crop.height)?;
    let sx = f64::from(width) / crop.width;
    let sy = f64::from(height) / crop.height;
    let color =
        CGColorSpace::new_device_rgb().ok_or("Core Graphics RGB color space unavailable")?;
    let mut pixels = vec![255_u8; width as usize * height as usize * 4];
    // The owned pixel buffer outlives the context and has the exact checked RGBA stride.
    let context = unsafe {
        CGBitmapContextCreate(
            pixels.as_mut_ptr().cast(),
            width as usize,
            height as usize,
            8,
            width as usize * 4,
            Some(&color),
            CGImageAlphaInfo::PremultipliedLast.0 | CGImageByteOrderInfo::Order32Big.0,
        )
    }
    .ok_or("Core Graphics bitmap allocation failed")?;
    let viewport = CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(w * sx, h * sy));
    CGContext::translate_ctm(Some(&context), -crop.x * sx, -crop.y * sy);
    CGContext::concat_ctm(
        Some(&context),
        CGPDFPage::drawing_transform(Some(&page), box_kind, viewport, 0, false),
    );
    CGContext::draw_pdf_page(Some(&context), Some(&page));
    drop(context);
    let mut bitmap = image::RgbaImage::from_raw(width, height, pixels)
        .ok_or("PDF bitmap dimensions are invalid")?;
    image::imageops::flip_vertical_in_place(&mut bitmap);
    let mut output = Cursor::new(Vec::new());
    bitmap
        .write_to(&mut output, image::ImageFormat::Png)
        .map_err(|error| error.to_string())?;
    Ok(super::PdfRaster {
        bytes: output.into_inner(),
        width,
        height,
    })
}

fn visible_bounds(page: &CGPDFPage) -> (CGRect, CGPDFBox) {
    let crop = CGPDFPage::box_rect(Some(page), CGPDFBox::CropBox);
    let media = CGPDFPage::box_rect(Some(page), CGPDFBox::MediaBox);
    let x = crop.origin.x.max(media.origin.x);
    let y = crop.origin.y.max(media.origin.y);
    let right = (crop.origin.x + crop.size.width).min(media.origin.x + media.size.width);
    let top = (crop.origin.y + crop.size.height).min(media.origin.y + media.size.height);
    if right <= x || top <= y {
        return (media, CGPDFBox::MediaBox);
    }
    (
        CGRect::new(CGPoint::new(x, y), CGSize::new(right - x, top - y)),
        CGPDFBox::CropBox,
    )
}
