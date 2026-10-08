use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use tauri::{ipc::Channel, AppHandle};
use tokio_util::sync::CancellationToken;

use crate::{
    ai::{
        contracts::speech_wire::{AlignmentProfile, InferenceRequestIdentity, SpeechAlignParams},
        local_runtime::{admission::Priority, device_profiles, speech},
    },
    local_resources::preparation,
    media::transcript::{MediaTranscript, TranscriptAlignmentProvenance},
    spaces::assets::media_analysis,
};

use super::{pipeline::audio_reference, MediaTranscriptionProgress};

const ENGINE_ID: &str = "qwen3_forced_aligner";
mod profile;
mod request;
mod units;

pub(crate) use profile::validate_language;

pub(crate) async fn align_media(
    app: &AppHandle,
    job_id: &str,
    source: &str,
    source_revision: &str,
    data: &mut serde_json::Value,
    progress: &Channel<MediaTranscriptionProgress>,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    profile::qualified_platform()?;
    let request = request::load(app, job_id).await?;
    let mut transcript: MediaTranscript =
        serde_json::from_value(data["mediaKnowledge"]["transcript"].clone())
            .map_err(|error| format!("Media transcript is invalid: {error}"))?;
    if transcript.revision != request.expected_revision
        || transcript.source_revision != source_revision
    {
        return Err("Transcript revision is obsolete".into());
    }
    let language = validate_language(&transcript.language)?;
    let selected_ids = request.segment_ids.iter().cloned().collect::<HashSet<_>>();
    let (selected, total_duration) = request::segments(&transcript, &request)?;

    super::send_progress(progress, "preparing", 3);
    let app_for_audio = app.clone();
    let source_path = source.to_owned();
    let revision = source_revision.to_owned();
    let token = cancellation.clone();
    let audio = crate::blocking::run(move || {
        media_analysis::prepare(&app_for_audio, Path::new(&source_path), &revision, &token)
    })
    .await?;
    let resources = preparation::prepare_models_for_job(
        app,
        request.model.preparation(),
        &format!("resource:{job_id}"),
        "Preparing forced alignment",
        cancellation,
        &mut |resources| {
            super::observe(
                progress,
                MediaTranscriptionProgress {
                    stage: "preparingResources".into(),
                    progress: 8,
                    partial: None,
                    resources: Some(resources),
                },
            );
        },
    )
    .await
    .map_err(|error| {
        if cancellation.is_cancelled() || error == preparation::RESOURCE_PREPARATION_CANCELLED {
            "cancelled".to_owned()
        } else {
            format!("local-resource-unavailable: {error}")
        }
    })?;
    let audio_ref = crate::blocking::run({
        let wav = audio.wav.clone();
        move || audio_reference(&wav)
    })
    .await?;
    let deadline_ms = profile::deadline(total_duration)?;
    let handle = app.clone();
    let snapshot = request
        .model
        .snapshot()
        .ok_or("Forced alignment model snapshot is unavailable")?;
    let runtime = crate::blocking::run(move || {
        crate::local_resources::runtimes::python::resolve(&handle, snapshot.runtime_profile())?
            .ok_or_else(|| "runtime_mismatch: prepared alignment runtime is missing".to_owned())
    })
    .await?;
    let device =
        device_profiles::alignment::profile(&runtime, request.model, cancellation, deadline_ms)
            .await?;
    let model_spec_id = request.model.alignment_spec()?.to_owned();
    let execution_device = device.device.clone();
    let request_id = uuid::Uuid::new_v4().to_string();
    let alignment = speech::align(
        app,
        &resources,
        device,
        SpeechAlignParams {
            identity: InferenceRequestIdentity {
                request_id: request_id.clone(),
                owner_kind: "media_alignment".into(),
                owner_id: job_id.into(),
                attempt_id: request_id,
                attempt_epoch: u64::from(request.expected_revision),
                input_revision: format!(
                    "{source_revision}:transcript:{}",
                    request.expected_revision
                ),
                model_spec_id: model_spec_id.clone(),
                deadline_ms,
            },
            audio_ref,
            text_segments: selected,
            profile: AlignmentProfile {
                language: language.into(),
                device: execution_device,
            },
        },
        cancellation,
        Priority::Background,
    )
    .await?;
    super::send_progress(progress, "aligning", 92);

    let mut by_segment = HashMap::<String, Vec<_>>::new();
    for unit in alignment.units {
        by_segment
            .entry(unit.segment_id.clone())
            .or_default()
            .push(unit);
    }
    for segment in &mut transcript.segments {
        if !selected_ids.contains(&segment.id) {
            continue;
        }
        segment.units = units::map_units(
            &segment.id,
            &segment.text,
            segment.start_ms,
            segment.end_ms,
            by_segment.remove(&segment.id).unwrap_or_default(),
        )?;
    }
    transcript.revision = transcript
        .revision
        .checked_add(1)
        .ok_or("Transcript revision exceeds the supported range")?;
    transcript.alignment = Some(TranscriptAlignmentProvenance {
        engine_id: ENGINE_ID.into(),
        model_spec_id: model_spec_id.clone(),
        model_revision: request.model.alignment_revision()?.into(),
        aligned_at: u64::try_from(crate::clock::unix_time_millis()?)
            .map_err(|error| error.to_string())?,
    });
    data["mediaKnowledge"]["transcript"] =
        serde_json::to_value(transcript).map_err(|error| error.to_string())?;
    super::send_progress(progress, "done", 100);
    Ok(())
}
