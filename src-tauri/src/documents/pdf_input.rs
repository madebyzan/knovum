use serde::{Deserialize, Serialize};

/// Display-frame PDF points: visible box after /Rotate, lower-left origin, y upward.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PdfRegion {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl PdfRegion {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if ![self.x, self.y, self.width, self.height]
            .into_iter()
            .all(f64::is_finite)
            || self.width <= 0.0
            || self.height <= 0.0
        {
            return Err("PDF region must have finite coordinates and positive dimensions".into());
        }
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PdfRecognitionReason {
    Scanned,
    SuspectedGarbledText,
    VectorText,
}

impl PdfRecognitionReason {
    pub(crate) fn from_parser(reason: Option<&str>) -> Result<Option<Self>, String> {
        match reason {
            Some(pdf_inspector::OCR_REASON_SCANNED) => Ok(Some(Self::Scanned)),
            Some(pdf_inspector::OCR_REASON_SUSPECTED_GARBLED_TEXT) => {
                Ok(Some(Self::SuspectedGarbledText))
            }
            Some(pdf_inspector::OCR_REASON_VECTOR_TEXT) => Ok(Some(Self::VectorText)),
            None | Some(pdf_inspector::OCR_REASON_NO_TEXT) => Ok(None),
            Some(value) => Err(format!("Unsupported PDF recognition reason: {value}")),
        }
    }
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum PdfInput {
    Page {
        page: u32,
        reason: PdfRecognitionReason,
    },
    Image {
        page: u32,
        rect: PdfRegion,
    },
}

impl PdfInput {
    pub(crate) fn page(&self) -> u32 {
        match self {
            Self::Page { page, .. } | Self::Image { page, .. } => *page,
        }
    }
    pub(crate) fn region(&self) -> Option<PdfRegion> {
        match self {
            Self::Image { rect, .. } => Some(*rect),
            Self::Page { .. } => None,
        }
    }
    pub(crate) fn location(&self) -> serde_json::Value {
        // Citation navigation is page-level. Raster geometry has its own explicit point frame,
        // rather than overloading the generic locator's historical rectangle convention.
        serde_json::json!({"kind":"pdf", "page":self.page()})
    }
}
