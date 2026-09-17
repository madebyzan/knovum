#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use crate::documents::pdf_input::{PdfInput, PdfRegion};

pub(crate) struct PdfRaster {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Background visual input, independent of canvas previews and presentation caches.
pub(crate) fn render(source: &std::path::Path, input: &PdfInput) -> Result<PdfRaster, String> {
    let page = input.page();
    if page == 0 {
        return Err("PDF page numbers start at one".into());
    }
    #[cfg(target_os = "macos")]
    {
        macos::render(source, page, input.region())
    }
    #[cfg(target_os = "windows")]
    {
        windows::render(source, page, input.region())
            .map_err(|error| format!("Windows PDF rendering failed: {error}"))
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = source;
        Err("Native PDF processing is not supported on this platform".into())
    }
}

fn viewport(width: f64, height: f64, region: Option<PdfRegion>) -> Result<PdfRegion, String> {
    let page = PdfRegion {
        x: 0.0,
        y: 0.0,
        width,
        height,
    };
    page.validate()?;
    let Some(region) = region else {
        return Ok(page);
    };
    region.validate()?;
    let x = region.x.max(0.0);
    let y = region.y.max(0.0);
    let clipped = PdfRegion {
        x,
        y,
        width: (region.x + region.width).min(width) - x,
        height: (region.y + region.height).min(height) - y,
    };
    clipped
        .validate()
        .map_err(|_| "PDF image region is outside the visible page".to_string())?;
    Ok(clipped)
}

fn dimensions(width: f64, height: f64) -> Result<(u32, u32), String> {
    if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
        return Err("PDF page dimensions are invalid".into());
    }
    let scale = (2048.0 / width.max(height)).min(2.0);
    Ok((
        (width * scale).ceil().max(1.0) as u32,
        (height * scale).ceil().max(1.0) as u32,
    ))
}
