mod reading;

use std::path::Path;

use image::{imageops::FilterType, DynamicImage, ImageReader, Limits, RgbaImage};
use serde_json::{json, Value};
use tauri::{ipc::Channel, AppHandle};
use tokio_util::sync::CancellationToken;

use super::RunEnrichmentJobRequest;
use crate::{
    ai::local_runtime::image::{compute::ImageTensor, text_recognition},
    foundation::{app_error::AppError, blocking},
    local_resources::{model_catalog::LocalModelId, preparation},
    spaces::{
        assets::{
            authorize_spaces_asset,
            enrichment::{commit, ensure_active, jobs::recognition_source},
            transcription::{send_progress, MediaTranscriptionProgress},
        },
        derivation::image_tensors,
        knowledge::{
            indexing::{self, KnowledgeChunkInput},
            validation::nodes::text_recognition::MAX_TEXT,
        },
    },
};

/// Past this a source is shrunk rather than refused: no text worth reading needs more pixels, and
/// the bound keeps every tensor inside what the worker admits.
const MAX_LONG_EDGE: u32 = 4096;

pub(super) async fn run(
    app: &AppHandle,
    request: &RunEnrichmentJobRequest,
    payload: &Value,
    progress: &Channel<MediaTranscriptionProgress>,
    cancellation: &CancellationToken,
) -> Result<Value, AppError> {
    let revision = request.source_revision.as_str();
    let job_id = request.job_id.as_str();
    let prepared = preparation::prepare_models_for_job(
        app,
        preparation::contracts::ResourcePreparationTarget::Model {
            model: LocalModelId::PpOcrV6Small,
        },
        &format!("resource:{job_id}"),
        "Preparing text recognition",
        cancellation,
        &mut |_| {},
    )
    .await
    .map_err(|error| format!("local-resource-unavailable: {error}"))?;
    let sources = recognition_source::sources(payload);
    let mut chunks = Vec::new();
    let mut recognized = String::new();
    for (index, source) in sources.iter().enumerate() {
        ensure_active(cancellation)?;
        let path = authorize_spaces_asset(app, Path::new(&source.path))?;
        let tensor = blocking::run(move || read_source(&path)).await?;
        let lines =
            text_recognition::recognize(app, job_id, revision, tensor, &prepared, cancellation)
                .await?;
        let text = reading::text(&lines);
        if !text.is_empty() {
            chunks.extend(
                indexing::chunks(
                    source.kind,
                    &text,
                    revision,
                    source.location.clone(),
                    json!({ "role": "recognized_text" }),
                )
                .into_iter()
                .map(KnowledgeChunkInput::from),
            );
            append_bounded(&mut recognized, &text);
        }
        let done = (index + 1) * 100 / sources.len();
        send_progress(progress, "recognizingText", done as u8);
    }
    let mut record = json!({
        "sourceRevision": revision,
        "modelSpecId": text_recognition::spec_id(),
        "generatedAt": crate::foundation::clock::unix_time_millis()?,
    });
    if !recognized.is_empty() {
        record["text"] = Value::from(recognized);
    }
    commit::publish_text_recognition(
        app,
        (&request.space_id, &request.node_id),
        commit::PublicationGuard {
            source_revision: Some(revision),
            job_id: Some(job_id),
            media_knowledge: None,
            cancellation: Some(cancellation),
        },
        record,
        chunks,
    )
    .await
}

/// The node keeps what its inspector can show; the whole text lives in the knowledge chunks, because
/// a record that grows with the document would be paid for on every read of the node.
fn append_bounded(target: &mut String, text: &str) {
    let used = target.encode_utf16().count();
    let separator = if target.is_empty() { "" } else { "\n\n" };
    let mut budget = MAX_TEXT.saturating_sub(used + separator.len());
    if budget == 0 {
        return;
    }
    target.push_str(separator);
    for character in text.chars() {
        let width = character.len_utf16();
        if width > budget {
            return;
        }
        budget -= width;
        target.push(character);
    }
}

fn read_source(path: &Path) -> Result<ImageTensor, String> {
    let mut reader = ImageReader::open(path)
        .map_err(|error| error.to_string())?
        .with_guessed_format()
        .map_err(|error| error.to_string())?;
    let mut limits = Limits::default();
    limits.max_alloc = Some(512 * 1024 * 1024);
    reader.limits(limits);
    let source = reader.decode().map_err(|error| error.to_string())?;
    let bounded = if source.width().max(source.height()) > MAX_LONG_EDGE {
        source.resize(MAX_LONG_EDGE, MAX_LONG_EDGE, FilterType::Triangle)
    } else {
        source
    };
    Ok(image_tensors::rgb_tensor(&on_paper(bounded)))
}

/// Transparent pixels hold arbitrary colour, and dark text on a transparent screenshot would vanish
/// into the black they usually carry. Text is read the way it is seen: on a white page.
fn on_paper(source: DynamicImage) -> RgbaImage {
    let mut image = source.into_rgba8();
    for pixel in image.pixels_mut() {
        let alpha = u32::from(pixel[3]);
        for channel in &mut pixel.0[..3] {
            *channel = ((u32::from(*channel) * alpha + 255 * (255 - alpha)) / 255) as u8;
        }
        pixel[3] = 255;
    }
    image
}

#[cfg(test)]
mod tests;
