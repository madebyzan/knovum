use std::path::Path;

use serde_json::Value;

use super::{capture, media, ExportFormats};
use crate::documents::generation::input::{admits, AuthoringContent};
use crate::documents::generation::DocumentFormat::{self, Docx, Html, Pdf, Pptx, Text};
use crate::documents::generation::MAX_AUTHORING_BYTES;
use crate::images::Encoding;

/// What an export produces. The requested file extension selects exactly one of
/// these, so choosing a format never needs a second instruction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Export {
    /// The same content, serialized from the resource by the document engine.
    Document(DocumentFormat),
    /// A rendering of the managed asset bytes.
    Managed(Managed),
    /// The summary a capture holds, as the engine assembles it.
    Summary,
}

pub(super) type ExportOffer = (Vec<(String, Export)>, Option<String>);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Managed {
    /// The bytes as they are.
    Original,
    /// The picture as it is currently edited, encoded again.
    Image(Encoding),
    /// The same recording, encoded again by FFmpeg.
    Media(&'static media::Target),
}

/// Every document source carries the same content, so it reaches every serialization the engine
/// can write - as long as the engine can read that content.
const DOCUMENT_TARGETS: &[DocumentFormat] = &[Text, Html, Docx, Pdf, Pptx];

/// A listing is the characters of a file, set as a document. Slides are not a listing.
const LISTING_TARGETS: &[DocumentFormat] = &[Text, Html, Docx, Pdf];

/// Every encoding a picture can be saved in. The picture is one render; these are its spellings.
const PICTURE_ENCODINGS: [Encoding; 4] = [
    Encoding::Png,
    Encoding::Jpeg,
    Encoding::WebP,
    Encoding::Tiff,
];

/// Why a resource that is still being prepared has no file yet.
const UNSETTLED: &str = "Wait for this resource to finish before exporting it";

/// The formats of one resource, as the person asking is told them.
pub(super) fn of(payload: &Value) -> Result<ExportFormats, String> {
    let (routes, unavailable) = offered(payload)?;
    Ok(ExportFormats {
        formats: routes.into_iter().map(|(extension, _)| extension).collect(),
        unavailable,
    })
}

/// The one export the destination asks for, or why this resource cannot produce it.
pub(super) fn requested(payload: &Value, destination: &Path) -> Result<Export, String> {
    let (routes, unavailable) = offered(payload)?;
    chosen(routes, unavailable, destination)
}

/// A picture a document carries is the PNG the Space extracted from it, untouched: it is offered
/// as it is, then in every other encoding a picture can be written in.
pub(super) fn document_resource() -> Vec<(String, Managed)> {
    deduplicated(picture_routes("png".into()))
}

/// The route among `routes` that the destination's extension names.
pub(super) fn chosen<T>(
    routes: Vec<(String, T)>,
    unavailable: Option<String>,
    destination: &Path,
) -> Result<T, String> {
    let extension = destination
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or("The export destination needs a file extension")?;
    // The two spellings of JPEG name one format.
    let requested = if extension == "jpeg" {
        "jpg"
    } else {
        &extension
    };
    routes
        .into_iter()
        .find(|(offered, _)| offered == requested)
        .map(|(_, route)| route)
        .ok_or_else(|| {
            unavailable
                .unwrap_or_else(|| format!("This resource cannot be exported as .{extension}"))
        })
}

/// What the engine offers for this resource, and why it offers nothing when that is the answer.
///
/// Nothing-to-offer is an answer, not a failure: a resource still being prepared and a document
/// the engine cannot read both report their reason here, so the person asking is told the same way
/// in both cases. Only a payload that cannot be read at all fails.
fn offered(payload: &Value) -> Result<ExportOffer, String> {
    let kind = payload
        .get("kind")
        .and_then(Value::as_str)
        .ok_or("The resource kind is unavailable")?;
    if unsettled(payload) {
        return Ok((vec![], Some(UNSETTLED.into())));
    }
    let routes = match kind {
        "image" => image_routes(payload)?,
        "audio" | "video" => media_routes(payload, &media::targets(kind))?,
        "file" => file_routes(payload)?,
        "code" => code_routes(payload)?,
        "note" | "markdown" | "text" => return editable_routes(kind, payload),
        "website" => return capture_routes(payload),
        // A credential and a Portal are references rather than content, and an object is
        // structured data: the Renderer serializes what they hold, so the engine offers nothing.
        _ => vec![],
    };
    Ok((deduplicated(routes), None))
}

/// A capture has no file, so its own format is the summary the engine assembles. That summary
/// stays available even when its text is outside the authoring contract, because writing it needs
/// no reading.
fn capture_routes(payload: &Value) -> Result<ExportOffer, String> {
    let own = ("md".to_owned(), Export::Summary);
    let summary = capture::summary(payload)?;
    Ok(match admits(AuthoringContent::Markdown(&summary)) {
        Ok(()) => (
            std::iter::once(own)
                .chain(document_routes(DOCUMENT_TARGETS))
                .collect(),
            None,
        ),
        Err(reason) => (vec![own], Some(reason)),
    })
}

/// A code resource is a text file: its own bytes, and a listing of them the document writers can
/// set. The listing is offered where the Space has already read the file as text and it fits the
/// authoring budget, so the offer rests on evidence rather than on hope.
fn code_routes(payload: &Value) -> Result<Vec<(String, Export)>, String> {
    let own = (
        source_extension(payload)?,
        Export::Managed(Managed::Original),
    );
    let readable = payload.get("previewText").and_then(Value::as_str).is_some()
        && payload
            .get("size")
            .and_then(Value::as_u64)
            .is_none_or(|size| size <= MAX_AUTHORING_BYTES as u64);
    if !readable {
        return Ok(vec![own]);
    }
    Ok(std::iter::once(own)
        .chain(document_routes(LISTING_TARGETS))
        .collect())
}

/// An editable resource reaches every serialization or none of them: each format reads the same
/// authoring model, so content the engine cannot read is unavailable in all of them.
fn editable_routes(kind: &str, payload: &Value) -> Result<ExportOffer, String> {
    let content = if kind == "markdown" {
        AuthoringContent::Markdown(
            payload
                .get("text")
                .and_then(Value::as_str)
                .ok_or("The document content is unavailable")?,
        )
    } else {
        AuthoringContent::Note(
            payload
                .get("tiptapContent")
                .ok_or("The document content is unavailable")?,
        )
    };
    Ok(match admits(content) {
        Ok(()) => (document_routes(DOCUMENT_TARGETS), None),
        Err(reason) => (vec![], Some(reason)),
    })
}

/// A depth field is measurement data rather than a picture, so only its own
/// 16-bit bytes carry its meaning. An edited picture has to be encoded again;
/// an untouched one is also available exactly as it was imported.
fn image_routes(payload: &Value) -> Result<Vec<(String, Export)>, String> {
    let original = source_extension(payload)?;
    let routes = if payload.get("imageDepth").is_some() {
        vec![(original, Managed::Original)]
    } else if payload.get("imageEdit").is_some() {
        encoding_routes().collect()
    } else {
        picture_routes(original)
    };
    Ok(routes
        .into_iter()
        .map(|(extension, managed)| (extension, Export::Managed(managed)))
        .collect())
}

/// A picture exactly as stored, then every encoding it can be written in.
fn picture_routes(original: String) -> Vec<(String, Managed)> {
    std::iter::once((original, Managed::Original))
        .chain(encoding_routes())
        .collect()
}

fn encoding_routes() -> impl Iterator<Item = (String, Managed)> {
    PICTURE_ENCODINGS
        .into_iter()
        .map(|encoding| (encoding.extension().to_owned(), Managed::Image(encoding)))
}

fn media_routes(
    payload: &Value,
    targets: &[&'static media::Target],
) -> Result<Vec<(String, Export)>, String> {
    Ok([(
        source_extension(payload)?,
        Export::Managed(Managed::Original),
    )]
    .into_iter()
    .chain(targets.iter().copied().map(|target| {
        (
            target.extension.to_owned(),
            Export::Managed(Managed::Media(target)),
        )
    }))
    .collect())
}

fn file_routes(payload: &Value) -> Result<Vec<(String, Export)>, String> {
    let extension = source_extension(payload)?;
    Ok([(extension.clone(), Export::Managed(Managed::Original))]
        .into_iter()
        .chain(document_routes(convertible(&extension)))
        .collect())
}

fn document_routes(formats: &[DocumentFormat]) -> Vec<(String, Export)> {
    formats
        .iter()
        .map(|format| (format.extension().to_owned(), Export::Document(*format)))
        .collect()
}

/// Supported semantic content, as owned by `documents/generation`. Slides are only slides when
/// the source was a presentation, and a PDF the engine read page by page is already a PDF.
fn convertible(extension: &str) -> &'static [DocumentFormat] {
    match extension {
        "ppt" => DOCUMENT_TARGETS,
        "pdf" => &[Text, Html, Docx, Pptx],
        "doc" | "docx" | "xls" | "xlsx" | "pptx" => &[Text, Html, Docx, Pdf],
        _ => &[],
    }
}

/// A resource the Space is still preparing has no settled content to save: an asset being
/// imported or processed, a derivation still running, a capture still being read.
fn unsettled(payload: &Value) -> bool {
    payload.get("processingState").is_some()
        || payload.get("derivationState").is_some()
        || payload.get("metadataState").and_then(Value::as_str) == Some("loading")
}

fn source_extension(payload: &Value) -> Result<String, String> {
    let name = payload
        .get("fileName")
        .and_then(Value::as_str)
        .ok_or("The resource file name is unavailable")?;
    Path::new(name)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| "The resource file name carries no format".into())
}

fn deduplicated<T>(routes: Vec<(String, T)>) -> Vec<(String, T)> {
    let mut offered = Vec::with_capacity(routes.len());
    for route in routes {
        if !offered
            .iter()
            .any(|(extension, _): &(String, T)| *extension == route.0)
        {
            offered.push(route);
        }
    }
    offered
}
