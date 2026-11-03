#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use std::path::Path;

use crate::documents::pdf_input::{PdfInput, PdfRegion};

pub(crate) struct PdfRaster {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// An opened PDF document that rasterizes pages and regions without re-opening or re-parsing the
/// source on every occurrence.
pub(crate) struct PdfRenderer {
    #[cfg(target_os = "macos")]
    inner: macos::Renderer,
    #[cfg(target_os = "windows")]
    inner: windows::Renderer,
}

impl PdfRenderer {
    pub(crate) fn open(source: &Path) -> Result<Self, String> {
        #[cfg(target_os = "macos")]
        {
            Ok(Self {
                inner: macos::Renderer::open(source)?,
            })
        }
        #[cfg(target_os = "windows")]
        {
            Ok(Self {
                inner: windows::Renderer::open(source)
                    .map_err(|error| format!("Windows PDF rendering failed: {error}"))?,
            })
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            let _ = source;
            Err("Native PDF processing is not supported on this platform".into())
        }
    }

    pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > 128 * 1024 * 1024 {
            return Err("PDF snapshot byte budget is invalid".into());
        }
        #[cfg(target_os = "macos")]
        {
            Ok(Self {
                inner: macos::Renderer::from_bytes(bytes)?,
            })
        }
        #[cfg(target_os = "windows")]
        {
            Ok(Self {
                inner: windows::Renderer::from_bytes(bytes)
                    .map_err(|error| format!("Windows PDF rendering failed: {error}"))?,
            })
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            Err("Native PDF processing is not supported on this platform".into())
        }
    }

    pub(crate) fn render(&self, input: &PdfInput) -> Result<PdfRaster, String> {
        let page = input.page();
        if page == 0 {
            return Err("PDF page numbers start at one".into());
        }
        #[cfg(target_os = "macos")]
        {
            self.inner.render(page, input.region())
        }
        #[cfg(target_os = "windows")]
        {
            self.inner
                .render(page, input.region())
                .map_err(|error| format!("Windows PDF rendering failed: {error}"))
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            Err("Native PDF processing is not supported on this platform".into())
        }
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
