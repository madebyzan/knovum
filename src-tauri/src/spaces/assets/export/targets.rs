use std::path::Path;

use serde_json::Value;

use super::{image::Encoding, media, ExportFormats};
use crate::documents::generation::input::{admits, AuthoringContent};
use crate::documents::generation::DocumentFormat::{self, Docx, Html, Pdf, Pptx, Text};

/// What an export produces. The requested file extension selects exactly one of
/// these, so choosing a format never needs a second instruction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Export {
    /// The same content, serialized from the resource by the document engine.
    Document(DocumentFormat),
    /// A rendering of the managed asset bytes.
    Managed(Managed),
}

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
    let (routes, unavailable) = offered(payload)?;
    routes
        .into_iter()
        .find(|(offered, _)| offered == requested)
        .map(|(_, export)| export)
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
fn offered(payload: &Value) -> Result<(Vec<(String, Export)>, Option<String>), String> {
    let kind = payload
        .get("kind")
        .and_then(Value::as_str)
        .ok_or("The resource kind is unavailable")?;
    if unsettled(payload) {
        return Ok((vec![], Some(UNSETTLED.into())));
    }
    let routes = match kind {
        "image" => image_routes(payload)?,
        "audio" | "video" => media_routes(payload, media::targets(kind))?,
        "file" => file_routes(payload)?,
        "note" | "markdown" | "text" => return editable_routes(kind, payload),
        // A website summary, an object or a credential is serialized from the
        // content the Renderer already holds, so the engine offers nothing.
        _ => vec![],
    };
    Ok((deduplicated(routes), None))
}

/// An editable resource reaches every serialization or none of them: each format reads the same
/// authoring model, so content the engine cannot read is unavailable in all of them.
fn editable_routes(
    kind: &str,
    payload: &Value,
) -> Result<(Vec<(String, Export)>, Option<String>), String> {
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
    let original = (
        source_extension(payload)?,
        Export::Managed(Managed::Original),
    );
    if payload.get("imageDepth").is_some() {
        return Ok(vec![original]);
    }
    let encodings = [Encoding::Png, Encoding::Jpeg].map(|encoding| {
        (
            encoding.extension().to_owned(),
            Export::Managed(Managed::Image(encoding)),
        )
    });
    if payload.get("imageEdit").is_some() {
        return Ok(encodings.into());
    }
    Ok([original].into_iter().chain(encodings).collect())
}

fn media_routes(
    payload: &Value,
    targets: &'static [media::Target],
) -> Result<Vec<(String, Export)>, String> {
    Ok([(
        source_extension(payload)?,
        Export::Managed(Managed::Original),
    )]
    .into_iter()
    .chain(targets.iter().map(|target| {
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

fn unsettled(payload: &Value) -> bool {
    payload.get("processingState").is_some() || payload.get("derivationState").is_some()
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

fn deduplicated(routes: Vec<(String, Export)>) -> Vec<(String, Export)> {
    let mut offered = Vec::with_capacity(routes.len());
    for route in routes {
        if !offered
            .iter()
            .any(|(extension, _): &(String, Export)| *extension == route.0)
        {
            offered.push(route);
        }
    }
    offered
}
