use crate::ai::local_runtime::compute_budget::ComputeBudget;
use std::{
    collections::{BTreeMap, HashMap},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use sqlx::Row;
use tauri::{ipc::Channel, AppHandle};
use tokio_util::sync::CancellationToken;

use crate::{
    ai::{
        contracts::{
            runtime::RuntimeDeviceProfile,
            speech_wire::{
                InferenceRequestIdentity, SpeakerCountBounds, SpeakerTurn, SpeechDiarizeParams,
            },
        },
        local_runtime::{admission::Priority, speech},
    },
    local_resources::{
        model_catalog::LocalModelId,
        preparation::{self, contracts::ResourcePreparationTarget, PreparedDiarizationModel},
    },
    media::transcript::{
        MediaTranscript, SpeakerTrack as TranscriptSpeakerTrack,
        SpeakerTurn as TranscriptSpeakerTurn, TranscriptDiarizationProvenance,
    },
    spaces::{assets::media_analysis, db},
};

use super::{pipeline::audio_reference, MediaTranscriptionProgress};

struct Request {
    model: LocalModelId,
    expected_revision: u32,
    minimum: Option<u16>,
    maximum: Option<u16>,
}

pub(crate) async fn diarize_media(
    app: &AppHandle,
    job_id: &str,
    source: &str,
    source_revision: &str,
    payload: &mut serde_json::Value,
    progress: &Channel<MediaTranscriptionProgress>,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let request = load_request(app, job_id).await?;
    let transcript: MediaTranscript =
        serde_json::from_value(payload["mediaKnowledge"]["transcript"].clone())
            .map_err(|error| format!("Media transcript is invalid: {error}"))?;
    if transcript.revision != request.expected_revision
        || transcript.source_revision != source_revision
    {
        return Err("Transcript revision is obsolete".into());
    }
    if transcript.duration_ms == 0 || transcript.duration_ms > 2 * 60 * 60 * 1_000 {
        return Err("Diarization media must not exceed two hours".into());
    }
    super::send_progress(progress, "preparing", 5);
    let app_for_audio = app.clone();
    let source_path = source.to_owned();
    let revision = source_revision.to_owned();
    let token = cancellation.clone();
    let audio = crate::foundation::blocking::run(move || {
        media_analysis::prepare(&app_for_audio, Path::new(&source_path), &revision, &token)
    })
    .await?;
    let variant = request.model.diarization_variant()?;
    let models = preparation::prepare_models_for_job(
        app,
        ResourcePreparationTarget::Model {
            model: request.model,
        },
        &format!("resource:{job_id}"),
        "Preparing speaker recognition resources",
        cancellation,
        &mut |resources| {
            let update = MediaTranscriptionProgress {
                stage: "preparingResources".into(),
                progress: 0,
                partial: None,
                resources: Some(resources),
            };
            super::observe(progress, update);
        },
    )
    .await?;
    let model = PreparedDiarizationModel::from_models(models, variant)?;
    let audio_ref = crate::foundation::blocking::run({
        let wav = audio.wav.clone();
        move || audio_reference(&wav)
    })
    .await?;
    let request_id = uuid::Uuid::new_v4().to_string();
    let diarization = speech::diarize(
        app,
        &model,
        RuntimeDeviceProfile {
            device: "cpu".into(),
            execution_profile_id: variant.spec_id().into(),
            cpu_threads: ComputeBudget::detect()?.model_threads,
        },
        SpeechDiarizeParams {
            identity: InferenceRequestIdentity {
                request_id: request_id.clone(),
                owner_kind: "media_diarization".into(),
                owner_id: job_id.into(),
                attempt_id: request_id,
                attempt_epoch: u64::from(request.expected_revision),
                input_revision: format!(
                    "{source_revision}:transcript:{}",
                    request.expected_revision
                ),
                model_spec_id: variant.spec_id().into(),
                deadline_ms: deadline(transcript.duration_ms)?,
            },
            audio_ref,
            speaker_count_bounds: SpeakerCountBounds {
                minimum: request.minimum,
                maximum: request.maximum,
            },
        },
        cancellation,
        Priority::Background,
    )
    .await?;
    if diarization
        .turns
        .iter()
        .any(|turn| turn.end_ms > transcript.duration_ms)
    {
        return Err("invalid_output: diarization turn exceeds media duration".into());
    }
    publish(transcript, diarization.turns, variant, payload)?;
    super::send_progress(progress, "done", 100);
    Ok(())
}

fn publish(
    mut transcript: MediaTranscript,
    turns: Vec<SpeakerTurn>,
    variant: crate::local_resources::models::catalog::DiarizationVariant,
    payload: &mut serde_json::Value,
) -> Result<(), String> {
    let mut tracks = BTreeMap::<String, Vec<&SpeakerTurn>>::new();
    for turn in &turns {
        tracks
            .entry(turn.speaker_id.clone())
            .or_default()
            .push(turn);
    }
    for segment in &mut transcript.segments {
        let duration = segment
            .end_ms
            .checked_sub(segment.start_ms)
            .filter(|duration| *duration > 0)
            .ok_or("Transcript segment timing is invalid")?;
        let mut overlap = HashMap::<&str, u64>::new();
        for turn in &turns {
            let amount = segment
                .end_ms
                .min(turn.end_ms)
                .saturating_sub(segment.start_ms.max(turn.start_ms));
            if amount > 0 {
                *overlap.entry(&turn.speaker_id).or_default() += amount;
            }
        }
        let mut ranked = overlap.into_iter().collect::<Vec<_>>();
        ranked.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(right.0)));
        let speaker = ranked
            .first()
            .filter(|winner| {
                winner.1.saturating_mul(2) > duration
                    && ranked.get(1).is_none_or(|runner_up| runner_up.1 < winner.1)
            })
            .map(|winner| winner.0);
        segment.speaker_id = speaker.map(str::to_owned);
    }
    transcript.revision = transcript
        .revision
        .checked_add(1)
        .ok_or("Transcript revision exceeds its range")?;
    transcript.speaker_tracks = tracks
        .into_iter()
        .map(|(speaker_id, turns)| TranscriptSpeakerTrack {
            speaker_id,
            turns: turns
                .into_iter()
                .map(|turn| TranscriptSpeakerTurn {
                    start_ms: turn.start_ms,
                    end_ms: turn.end_ms,
                })
                .collect(),
        })
        .collect();
    transcript.diarization = Some(TranscriptDiarizationProvenance {
        engine_id: "sherpa_onnx".into(),
        model_spec_id: variant.spec_id().into(),
        model_revision: variant.revision().into(),
        diarized_at: u64::try_from(crate::foundation::clock::unix_time_millis()?)
            .map_err(|error| error.to_string())?,
    });
    payload["mediaKnowledge"]["transcript"] =
        serde_json::to_value(transcript).map_err(|error| error.to_string())?;
    Ok(())
}

async fn load_request(app: &AppHandle, job_id: &str) -> Result<Request, String> {
    let row = sqlx::query("SELECT model_spec_id,expected_transcript_revision,minimum_speakers,maximum_speakers FROM spaces_media_diarization_requests WHERE job_id=$1")
        .bind(job_id).fetch_optional(db::pool(app).await?).await.map_err(db::sql_error)?
        .ok_or("Diarization request payload is missing")?;
    Ok(Request {
        model: LocalModelId::from_diarization_spec(
            row.try_get::<&str, _>("model_spec_id")
                .map_err(db::sql_error)?,
        )?,
        expected_revision: u32::try_from(
            row.try_get::<i64, _>("expected_transcript_revision")
                .map_err(db::sql_error)?,
        )
        .map_err(|error| error.to_string())?,
        minimum: row
            .try_get::<Option<i64>, _>("minimum_speakers")
            .map_err(db::sql_error)?
            .map(u16::try_from)
            .transpose()
            .map_err(|error| error.to_string())?,
        maximum: row
            .try_get::<Option<i64>, _>("maximum_speakers")
            .map_err(db::sql_error)?
            .map(u16::try_from)
            .transpose()
            .map_err(|error| error.to_string())?,
    })
}

fn deadline(duration_ms: u64) -> Result<u64, String> {
    let now = u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_millis(),
    )
    .map_err(|error| error.to_string())?;
    now.checked_add(
        duration_ms
            .saturating_mul(4)
            .clamp(120_000, 8 * 60 * 60 * 1_000),
    )
    .ok_or("Diarization deadline exceeds its range".into())
}
