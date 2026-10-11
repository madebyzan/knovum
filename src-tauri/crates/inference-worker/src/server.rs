use crate::{
    artifact,
    cancellation::{ActiveRequest, Cancellation},
    initialization::{Engine, Session},
    protocol::{self, Output, Request},
};
use knovum_inference_contracts::{
    embedding_wire::EmbeddingEncodeParams, image_compute::ImageComputeParams,
    image_wire::ImageSegmentParams, runtime::*,
};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::{
    io::{self, BufRead},
    sync::{mpsc, Arc, Mutex},
    thread,
};

struct Job {
    request: Request,
    cancellation: Cancellation,
}

pub fn serve() -> Result<(), String> {
    let output = Output::new(io::stdout());
    let acknowledgements = Arc::new(crate::speech::synthesis::Acknowledgements::default());
    let compute_acknowledgements = acknowledgements.clone();
    let active = Arc::new(ActiveRequest::default());
    let session_id = Arc::new(Mutex::new(None::<String>));
    let (sender, receiver) = mpsc::sync_channel::<Job>(1);
    let (compute_output, compute_active, compute_session_id) =
        (output.clone(), active.clone(), session_id.clone());
    let compute = thread::spawn(move || -> Result<(), String> {
        let mut session = None;
        for job in receiver {
            let id = job
                .request
                .id
                .as_deref()
                .ok_or("invalid_input: compute request has no identifier")?;
            let result = execute(
                &mut session,
                &job.request,
                &job.cancellation,
                &compute_output,
                &compute_acknowledgements,
            );
            let result = job.cancellation.check().and(result);
            if let Some(session) = &session {
                *compute_session_id
                    .lock()
                    .map_err(|_| "worker_crashed: session lock poisoned")? =
                    Some(session.identity.session_id.clone());
            }
            compute_active.finish()?;
            match result {
                Ok(result) => compute_output.result(id, &result)?,
                Err(error) => compute_output.error(Some(id), &error)?,
            }
        }
        Ok(())
    });
    let result = read_requests(
        &mut io::stdin().lock(),
        &sender,
        &active,
        &session_id,
        &output,
        &acknowledgements,
    );
    if let Ok(active) = active.0.lock() {
        if let Some((_, _, cancel)) = active.as_ref() {
            cancel.cancel();
        }
    }
    drop(sender);
    let settled = compute
        .join()
        .map_err(|_| "worker_crashed: native compute thread panicked")?;
    result.and(settled)
}

fn read_requests(
    reader: &mut impl BufRead,
    sender: &mpsc::SyncSender<Job>,
    active: &ActiveRequest,
    session_id: &Mutex<Option<String>>,
    output: &Output,
    acknowledgements: &crate::speech::synthesis::Acknowledgements,
) -> Result<(), String> {
    while let Some(value) = protocol::read_frame(reader, MAX_MESSAGE_BYTES)? {
        let request: Request =
            serde_json::from_value(value).map_err(|error| format!("invalid_input: {error}"))?;
        if request.jsonrpc != "2.0"
            || request
                .id
                .as_ref()
                .is_some_and(|id| !artifact::valid_name(id, 128, false))
        {
            return Err("invalid_input: invalid JSON-RPC request identity".into());
        }
        match request.method.as_str() {
            "runtime.ackChunk" => {
                acknowledgements.acknowledge(&decode::<RuntimeAckChunkParams>(&request.params)?)?;
            }
            "runtime.cancel" => {
                let params: RuntimeCancelParams = decode(&request.params)?;
                let accepted = active.cancel(&params.target_request_id, params.attempt_epoch)?;
                if let Some(id) = &request.id {
                    output.result(id, &RuntimeCancellationAcknowledgement { accepted })?;
                }
            }
            "runtime.health" => {
                let params: RuntimeHealthParams = decode(&request.params)?;
                if session_id
                    .lock()
                    .map_err(|_| "worker_crashed: session lock poisoned")?
                    .as_deref()
                    != Some(params.session_id.as_str())
                {
                    output.error(
                        request.id.as_deref(),
                        "runtime_mismatch: health session differs",
                    )?;
                    continue;
                }
                let current = active
                    .0
                    .lock()
                    .map_err(|_| "worker_crashed: admission lock poisoned")?
                    .as_ref()
                    .map(|(id, _, _)| id.clone());
                let memory = memory_stats::memory_stats()
                    .ok_or("resource_exhausted: resident memory is unavailable")?;
                output.result(
                    request
                        .id
                        .as_deref()
                        .ok_or("invalid_input: health request has no identifier")?,
                    &RuntimeHealthObservation {
                        session_id: params.session_id,
                        current_request_id: current,
                        resident_bytes: memory.physical_mem as u64,
                    },
                )?;
            }
            "runtime.shutdown" => {
                let params: RuntimeShutdownParams = decode(&request.params)?;
                let idle = active
                    .0
                    .lock()
                    .map_err(|_| "worker_crashed: admission lock poisoned")?
                    .is_none();
                if !idle
                    || session_id
                        .lock()
                        .map_err(|_| "worker_crashed: session lock poisoned")?
                        .as_deref()
                        != Some(params.session_id.as_str())
                {
                    output.error(
                        request.id.as_deref(),
                        "runtime_mismatch: shutdown session or state differs",
                    )?;
                    continue;
                }
                output.result(
                    request
                        .id
                        .as_deref()
                        .ok_or("invalid_input: shutdown has no identifier")?,
                    &RuntimeShutdownAcknowledgement { accepted: true },
                )?;
                return Ok(());
            }
            _ => {
                let id = request
                    .id
                    .as_deref()
                    .ok_or("invalid_input: computation cannot be a notification")?;
                let epoch = request
                    .params
                    .get("attemptEpoch")
                    .and_then(Value::as_u64)
                    .unwrap_or(0);
                let deadline = if request.method == "runtime.initialize" {
                    now_ms()?.saturating_add(15_000)
                } else {
                    request
                        .params
                        .get("deadlineMs")
                        .and_then(Value::as_u64)
                        .ok_or("invalid_input: missing deadline")?
                };
                match active.admit(id, epoch, deadline) {
                    Err(error) => output.error(Some(id), &error)?,
                    Ok(cancellation) => {
                        if sender
                            .try_send(Job {
                                request,
                                cancellation,
                            })
                            .is_err()
                        {
                            active.finish()?;
                            return Err("worker_crashed: compute queue unavailable".into());
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn execute(
    session: &mut Option<Session>,
    request: &Request,
    cancel: &Cancellation,
    output: &Output,
    acknowledgements: &crate::speech::synthesis::Acknowledgements,
) -> Result<Value, String> {
    if request.method == "runtime.initialize" {
        if session.is_some() {
            return Err("runtime_mismatch: worker is already initialized".into());
        }
        let initialized = Session::initialize(decode(&request.params)?)?;
        let result =
            serde_json::to_value(&initialized.identity).map_err(|error| error.to_string())?;
        *session = Some(initialized);
        return Ok(result);
    }
    let session = session
        .as_mut()
        .ok_or("runtime_mismatch: worker is not initialized")?;
    if request.method != session.identity.capability.method()
        || request.params.get("requestId").and_then(Value::as_str) != request.id.as_deref()
        || request.params.get("modelSpecId").and_then(Value::as_str)
            != Some(session.identity.capability_spec_id.as_str())
    {
        return Err("runtime_mismatch: request identity or capability differs".into());
    }
    let result = match (&mut session.engine, request.method.as_str()) {
        (Engine::Images(images), "image.segment") => serde_json::to_value(images.segment(
            &session.work,
            decode::<ImageSegmentParams>(&request.params)?,
            cancel,
        )?),
        (
            Engine::Images(images),
            "image.upscale" | "image.inpaint" | "image.select" | "image.depth",
        ) => serde_json::to_value(images.compute(
            &session.work,
            decode::<ImageComputeParams>(&request.params)?,
            cancel,
        )?),
        (Engine::Embedding(embedding), "embedding.encode") => serde_json::to_value(
            embedding.encode(decode::<EmbeddingEncodeParams>(&request.params)?, cancel)?,
        ),
        (Engine::Transcription(transcriber), "speech.transcribe") => serde_json::to_value(
            transcriber.transcribe(&session.work, decode(&request.params)?, cancel, output)?,
        ),
        (Engine::Separation(separator), "speech.separate") => serde_json::to_value(
            separator.separate(&session.work, decode(&request.params)?, cancel)?,
        ),
        (Engine::Diarization(diarizer), "speech.diarize") => {
            serde_json::to_value(diarizer.diarize(decode(&request.params)?, cancel)?)
        }
        (Engine::Alignment(aligner), "speech.align") => {
            serde_json::to_value(aligner.align(decode(&request.params)?, cancel)?)
        }
        (Engine::Synthesis(synthesis), "speech.synthesize") => {
            serde_json::to_value(synthesis.synthesize(
                &session.work,
                decode(&request.params)?,
                cancel,
                output,
                acknowledgements,
            )?)
        }
        _ => return Err("runtime_mismatch: worker does not own requested method".into()),
    };
    result.map_err(|error| format!("invalid_output: {error}"))
}

fn decode<T: DeserializeOwned>(value: &Value) -> Result<T, String> {
    serde_json::from_value(value.clone()).map_err(|error| format!("invalid_input: {error}"))
}
fn now_ms() -> Result<u64, String> {
    Ok(std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_millis() as u64)
}
