use mime::Mime;
use reqwest::{header::CONTENT_TYPE, Response, Url};
use serde::Serialize;

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum WebDocumentKind {
    Html,
    Json,
    Xml,
    Text,
}

pub(crate) fn response_media_type(response: &Response) -> Option<Mime> {
    response
        .headers()
        .get(CONTENT_TYPE)?
        .to_str()
        .ok()?
        .parse()
        .ok()
}

pub(crate) fn classify_document(
    response_type: Option<&Mime>,
    url: &Url,
    bytes: &[u8],
) -> Option<WebDocumentKind> {
    response_type
        .and_then(kind_from_media_type)
        .or_else(|| {
            mime_guess::from_path(url.path())
                .first()
                .as_ref()
                .and_then(kind_from_media_type)
        })
        .or_else(|| infer_text_kind(bytes))
        .or_else(|| {
            std::str::from_utf8(bytes)
                .ok()
                .map(|_| WebDocumentKind::Text)
        })
}

pub(crate) fn detect_image_media_type(
    header: Option<Mime>,
    url: &Url,
    bytes: &[u8],
) -> Option<Mime> {
    if let Some(inferred) = infer::get(bytes) {
        return inferred.mime_type().parse::<Mime>().ok().filter(is_image);
    }
    header
        .filter(is_image)
        .or_else(|| mime_guess::from_path(url.path()).first().filter(is_image))
}

fn kind_from_media_type(media_type: &Mime) -> Option<WebDocumentKind> {
    let essence = media_type.essence_str();
    if essence == "text/html" || essence == "application/xhtml+xml" {
        return Some(WebDocumentKind::Html);
    }
    if media_type.subtype() == "json" || media_type.suffix().is_some_and(|suffix| suffix == "json")
    {
        return Some(WebDocumentKind::Json);
    }
    if media_type.subtype() == "xml" || media_type.suffix().is_some_and(|suffix| suffix == "xml") {
        return Some(WebDocumentKind::Xml);
    }
    (media_type.type_() == mime::TEXT).then_some(WebDocumentKind::Text)
}

fn infer_text_kind(bytes: &[u8]) -> Option<WebDocumentKind> {
    if infer::text::is_html(bytes) {
        Some(WebDocumentKind::Html)
    } else if infer::text::is_xml(bytes) {
        Some(WebDocumentKind::Xml)
    } else {
        None
    }
}

fn is_image(media_type: &Mime) -> bool {
    media_type.type_() == mime::IMAGE
}
