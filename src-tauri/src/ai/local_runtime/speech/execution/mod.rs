use crate::local_resources::runtimes::python;
use worker_start::{start_worker, WorkerStart};
mod worker_start;
use std::{
    path::Path,
    time::{Duration, Instant},
};

use serde::{de::DeserializeOwned, Serialize};
use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

use super::super::{admission::Priority, process::Worker};
use super::stream::{consume_synthesis_chunk, SynthesisChunkCallback, SynthesisStream};
use crate::ai::contracts::{
    runtime::{
        RuntimeAckChunkParams, RuntimeCancelParams, RuntimeCapability, RuntimeDeviceProfile,
    },
    speech_wire::{
        InferenceRequestIdentity, SpeechProgressEvent, SpeechProgressEventType,
        SpeechSynthesisChunkEvent,
    },
};

const CANCEL_GRACE: Duration = Duration::from_secs(5);

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
    fn method(&self) -> &'static str;
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
    let deadline = super::super::deadline_duration(identity.deadline_ms)?;
    let exit = super::super::exit_token();
    let runtime =
        python::resolve(app, python::RuntimeProfile::Inference)?.ok_or("runtime_missing")?;
    let capability = params.capability();
    super::super::prepare_worker_for_admission(priority, |worker| {
        worker.matches(
            &runtime.digest,
            model_path,
            capability,
            &identity.model_spec_id,
            &device_profile,
        )
    })
    .await?;
    let _permit = super::super::admission::acquire(priority, cancellation, exit, deadline).await?;
    let slot = super::super::worker_slot();
    let mut guard = tokio::select! {
        biased;
        _ = exit.cancelled() => return Err("cancelled".into()),
        _ = cancellation.cancelled() => return Err("cancelled".into()),
        guard = slot.lock() => guard,
    };
    if let Some(worker) = guard.as_mut() {
        if !worker.matches(
            &runtime.digest,
            model_path,
            capability,
            &identity.model_spec_id,
            &device_profile,
        ) {
            worker.terminate().await;
            *guard = None;
        }
    }
    let work_directory = if let Some(worker) = guard.as_ref() {
        worker.work_directory().to_owned()
    } else {
        start_worker(
            WorkerStart {
                app,
                runtime: &runtime,
                model_path,
                capability,
                model_spec_id: identity.model_spec_id.clone(),
                device_profile,
                deadline_ms: identity.deadline_ms,
                cancellation,
                exit,
            },
            &mut guard,
        )
        .await?
    };
    let worker = guard.as_mut().ok_or("worker_crashed")?;
    let request_id = identity.request_id.clone();
    let attempt_epoch = identity.attempt_epoch;
    let mut last_event_seq = 0;
    let mut synthesis_stream =
        (capability == RuntimeCapability::SpeechSynthesis).then(SynthesisStream::new);
    let deadline = match worker
        .send_request(
            &request_id,
            params.method(),
            &params,
            identity.deadline_ms,
            cancellation,
            exit,
        )
        .await
    {
        Ok(deadline) => deadline,
        Err(error) => {
            *guard = None;
            return Err(error);
        }
    };
    let response = tokio::select! {
        biased;
        _ = exit.cancelled() => {
            cancel(worker, &request_id, attempt_epoch).await;
            *guard = None;
            return Err("cancelled".into());
        }
        _ = cancellation.cancelled() => {
            cancel(worker, &request_id, attempt_epoch).await;
            *guard = None;
            return Err("cancelled".into());
        }
        result = tokio::time::timeout_at(
            deadline,
            async {
                loop {
                    let message = worker.protocol.read().await?;
                    let Some(method) = message.get("method").and_then(serde_json::Value::as_str) else {
                        break super::super::protocol::Protocol::decode_response(message, &request_id);
                    };
                    if message.get("jsonrpc").and_then(serde_json::Value::as_str) != Some("2.0")
                        || message.get("id").is_some()
                    {
                        return Err("invalid_output: worker notification envelope is invalid".into());
                    }
                    let value = message.get("params").cloned()
                        .ok_or("invalid_output: worker notification has no parameters")?;
                    match (capability, method) {
                        (RuntimeCapability::SpeechTranscription, "speech.progress") => {
                            let event: SpeechProgressEvent = serde_json::from_value(value)
                                .map_err(|error| format!("invalid_output: {error}"))?;
                            validate_progress_event(&event, &request_id, attempt_epoch, last_event_seq)?;
                            last_event_seq = event.seq;
                            if let Some(callback) = on_progress.as_deref_mut() {
                                callback(event);
                            }
                        }
                        (RuntimeCapability::SpeechSynthesis, "speech.chunk") => {
                            let event: SpeechSynthesisChunkEvent = serde_json::from_value(value)
                                .map_err(|error| format!("invalid_output: {error}"))?;
                            let consumed_bytes = consume_synthesis_chunk(
                                synthesis_stream.as_mut().ok_or("invalid_output: synthesis stream is missing")?,
                                &event,
                                &work_directory,
                                &request_id,
                                attempt_epoch,
                                on_synthesis_chunk.as_deref_mut(),
                            )?;
                            worker.protocol.notify(
                                "runtime.ackChunk",
                                &RuntimeAckChunkParams {
                                    request_id: request_id.clone(),
                                    attempt_epoch,
                                    seq: event.seq,
                                    consumed_bytes,
                                },
                            ).await?;
                        }
                        _ => return Err("invalid_output: unexpected speech notification".into()),
                    }
                }
            },
        ) => match result {
            Ok(value) => value,
            Err(_) => {
                cancel(worker, &request_id, attempt_epoch).await;
                *guard = None;
                return Err("deadline_exceeded".into());
            }
        }
    };
    worker.last_used = Instant::now();
    let output = match response {
        Ok(value) => finish(value, &work_directory, identity, synthesis_stream),
        Err(error) => Err(error),
    };
    if output.is_err() {
        worker.terminate().await;
        *guard = None;
        return output;
    }
    drop(guard);
    super::super::reclaim_when_idle(slot);
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

async fn cancel(worker: &mut Worker, request_id: &str, attempt_epoch: u64) {
    let _ = tokio::time::timeout(CANCEL_GRACE, async {
        worker
            .protocol
            .notify(
                "runtime.cancel",
                &RuntimeCancelParams {
                    target_request_id: request_id.into(),
                    attempt_epoch,
                },
            )
            .await?;
        worker
            .protocol
            .read_response_with_events::<serde_json::Value>(request_id, |method, value| {
                if method != "speech.progress" {
                    return Err(
                        "invalid_output: unexpected notification during cancellation".into(),
                    );
                }
                let event: SpeechProgressEvent = serde_json::from_value(value)
                    .map_err(|error| format!("invalid_output: {error}"))?;
                if event.request_id != request_id || event.attempt_epoch != attempt_epoch {
                    return Err("invalid_output: cancellation progress identity differs".into());
                }
                Ok(())
            })
            .await
    })
    .await;
    worker.terminate().await;
}
