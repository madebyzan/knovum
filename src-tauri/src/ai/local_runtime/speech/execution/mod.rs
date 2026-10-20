use std::path::Path;

use serde::{de::DeserializeOwned, Serialize};
use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

use super::super::{
    admission::Priority,
    lease::{RequestIdentity, WorkerLease},
    process::WorkerSpecification,
};
use super::stream::{consume_synthesis_chunk, SynthesisChunkCallback, SynthesisStream};
use crate::ai::contracts::{
    runtime::{RuntimeAckChunkParams, RuntimeCapability, RuntimeDeviceProfile, RuntimeModel},
    speech_wire::{
        InferenceRequestIdentity, SpeechProgressEvent, SpeechProgressEventType,
        SpeechSynthesisChunkEvent,
    },
};

pub(in crate::ai::local_runtime) struct Execution<'a, R: Runtime> {
    pub(in crate::ai::local_runtime) app: &'a AppHandle<R>,
    pub(in crate::ai::local_runtime) model_path: &'a Path,
    pub(in crate::ai::local_runtime) device_profile: RuntimeDeviceProfile,
    pub(in crate::ai::local_runtime) cancellation: &'a CancellationToken,
    pub(in crate::ai::local_runtime) priority: Priority,
}

pub(in crate::ai::local_runtime) trait SpeechRequest:
    Serialize
{
    type Response: DeserializeOwned;

    fn identity(&self) -> &InferenceRequestIdentity;
    fn capability(&self) -> RuntimeCapability;
}

pub(in crate::ai::local_runtime) async fn execute<R, P, O, F>(
    execution: Execution<'_, R>,
    params: P,
    mut on_progress: Option<&mut (dyn FnMut(SpeechProgressEvent) + Send)>,
    mut on_synthesis_chunk: Option<&mut SynthesisChunkCallback<'_>>,
    finish: F,
) -> Result<O, String>
where
    R: Runtime,
    P: SpeechRequest,
    F: FnOnce(
        P::Response,
        &Path,
        &InferenceRequestIdentity,
        Option<SynthesisStream>,
    ) -> Result<O, String>,
{
    let Execution {
        app,
        model_path,
        device_profile,
        cancellation,
        priority,
    } = execution;
    let identity = params.identity();
    let capability = params.capability();
    let mut lease = WorkerLease::acquire(
        app,
        priority,
        WorkerSpecification {
            capability,
            capability_spec_id: identity.model_spec_id.clone(),
            device_profile,
            model: RuntimeModel::Snapshot {
                path: model_path.to_string_lossy().into_owned(),
            },
        },
        cancellation,
    )
    .await?;
    let work_directory = lease.work_directory()?.to_owned();
    let request_id = identity.request_id.clone();
    let attempt_epoch = identity.attempt_epoch;
    let mut last_event_seq = 0;
    let mut synthesis_stream =
        (capability == RuntimeCapability::SpeechSynthesis).then(SynthesisStream::new);
    // A speech worker publishes what its capability publishes: transcription reports progress and
    // synthesis streams chunks whose credit must be acknowledged. Anything else is not this
    // protocol.
    let mut on_event = |method: &str,
                        published: serde_json::Value|
     -> Result<Option<RuntimeAckChunkParams>, String> {
        match (capability, method) {
            (RuntimeCapability::SpeechTranscription, "speech.progress") => {
                let event: SpeechProgressEvent = serde_json::from_value(published)
                    .map_err(|error| format!("invalid_output: {error}"))?;
                validate_progress_event(&event, &request_id, attempt_epoch, last_event_seq)?;
                last_event_seq = event.seq;
                if let Some(callback) = on_progress.as_deref_mut() {
                    callback(event);
                }
                Ok(None)
            }
            (RuntimeCapability::SpeechSynthesis, "speech.chunk") => {
                let event: SpeechSynthesisChunkEvent = serde_json::from_value(published)
                    .map_err(|error| format!("invalid_output: {error}"))?;
                let consumed_bytes = consume_synthesis_chunk(
                    synthesis_stream
                        .as_mut()
                        .ok_or("invalid_output: synthesis stream is missing")?,
                    &event,
                    &work_directory,
                    &request_id,
                    attempt_epoch,
                    on_synthesis_chunk.as_deref_mut(),
                )?;
                Ok(Some(RuntimeAckChunkParams {
                    request_id: request_id.clone(),
                    attempt_epoch,
                    seq: event.seq,
                    consumed_bytes,
                }))
            }
            _ => Err("invalid_output: unexpected speech notification".into()),
        }
    };
    let response = lease
        .request_with_events::<P::Response>(
            &RequestIdentity {
                request_id: &request_id,
                attempt_epoch,
                budget_ms: identity.budget_ms,
            },
            &params,
            cancellation,
            &mut on_event,
        )
        .await?;
    let output = finish(response, &work_directory, identity, synthesis_stream);
    if output.is_err() {
        lease.discard().await;
    }
    output
}

fn validate_progress_event(
    event: &SpeechProgressEvent,
    request_id: &str,
    attempt_epoch: u64,
    last_event_seq: u64,
) -> Result<(), String> {
    if event.request_id != request_id
        || event.attempt_epoch != attempt_epoch
        || event.seq != last_event_seq + 1
        || event.progress > 100
        || !matches!(
            event.stage.as_str(),
            "loading" | "transcribing" | "publishing"
        )
        || !matches!(
            (&event.event_type, &event.partial),
            (SpeechProgressEventType::Progress, None)
                | (SpeechProgressEventType::PartialTranscript, Some(_))
        )
        || event
            .language
            .as_ref()
            .is_some_and(|language| language.is_empty() || language.len() > 64)
        || event.partial.as_ref().is_some_and(|segment| {
            segment.text.trim().is_empty() || segment.end_ms <= segment.start_ms
        })
    {
        return Err("invalid_output: speech progress identity or stage differs".into());
    }
    Ok(())
}
