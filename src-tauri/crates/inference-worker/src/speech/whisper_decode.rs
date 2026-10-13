use super::{audio::Audio, ffi::whisper::*, transcription::Engine};
use crate::{cancellation::Cancellation, protocol::Output};
use knovum_inference_contracts::speech_wire::{
    InferenceRequestIdentity, LanguagePolicy, SpeechProgressEvent, SpeechProgressEventType,
    SpeechTranscribeParams, SpeechTranscriptSegment,
};
use std::{
    ffi::{c_void, CStr},
    ops::Range,
};

pub(super) fn decode(
    engine: &Engine,
    params: &SpeechTranscribeParams,
    audio: &Audio,
    language: &CStr,
    regions: &[Range<u64>],
    cancel: &Cancellation,
    output: &Output,
) -> Result<(String, Vec<SpeechTranscriptSegment>), String> {
    let (Some(first), Some(last)) = (regions.first(), regions.last()) else {
        let language = match params.profile.language_policy {
            LanguagePolicy::Detect => "und".into(),
            LanguagePolicy::Fixed => language.to_string_lossy().into_owned(),
        };
        return Ok((language, Vec::new()));
    };
    let language = super::whisper_language::select(
        engine,
        &audio.samples,
        &params.profile.language_policy,
        language,
        regions,
        cancel,
    )?;
    let mut options = unsafe {
        engine
            .api
            .whisper_full_default_params(whisper_sampling_strategy_WHISPER_SAMPLING_BEAM_SEARCH)
    };
    options.n_threads = i32::from(engine.threads);
    options.beam_search.beam_size = i32::from(params.profile.beam_size);
    options.language = language.as_ptr();
    options.no_context = false;
    options.token_timestamps = true;
    options.print_special = false;
    options.print_progress = false;
    options.print_realtime = false;
    options.print_timestamps = false;
    // Keep the original waveform: compressed VAD timelines do not cover token timestamps.
    options.vad = false;
    options.offset_ms = first.start as i32;
    options.duration_ms = (last.end - first.start) as i32;
    let mut observer = Observer {
        output: output.clone(),
        identity: params.identity.clone(),
        cancel: cancel.clone(),
        seq: 0,
        last: -1,
        error: None,
    };
    options.abort_callback = Some(abort);
    options.abort_callback_user_data = std::ptr::from_ref(cancel).cast_mut().cast();
    options.progress_callback = Some(progress);
    options.progress_callback_user_data = std::ptr::from_mut(&mut observer).cast();
    // C ABI callbacks are synchronous; both stack-owned userdata values remain live until full returns.
    let status = unsafe {
        engine.api.whisper_full_with_state(
            engine.context,
            engine.state,
            options,
            audio.samples.as_ptr(),
            audio.samples.len() as i32,
        )
    };
    if let Some(error) = observer.error {
        return Err(error);
    }
    cancel.check()?;
    if status != 0 {
        return Err(format!("transcription_failed: Whisper status {status}"));
    }
    let count = unsafe { engine.api.whisper_full_n_segments_from_state(engine.state) };
    if !(0..=100_000).contains(&count) {
        return Err("invalid_output: Whisper segment count exceeds its budget".into());
    }
    let id = unsafe { engine.api.whisper_full_lang_id_from_state(engine.state) };
    let language = unsafe { engine.api.whisper_lang_str(id) };
    if language.is_null() {
        return Err("invalid_output: Whisper returned no detected language".into());
    }
    let language = unsafe { CStr::from_ptr(language) }
        .to_str()
        .map_err(|error| format!("invalid_output: {error}"))?
        .to_owned();
    let mut segments = Vec::with_capacity(count as usize);
    for index in 0..count {
        let segment = super::whisper_timing::segment(engine, index, audio.frames(), &language)?;
        if regions
            .iter()
            .any(|region| segment.start_ms < region.end && segment.end_ms > region.start)
        {
            segments.push(segment);
        }
    }
    Ok((language, segments))
}
struct Observer {
    output: Output,
    identity: InferenceRequestIdentity,
    cancel: Cancellation,
    seq: u64,
    last: i32,
    error: Option<String>,
}
unsafe extern "C" fn abort(pointer: *mut c_void) -> bool {
    unsafe { &*pointer.cast::<Cancellation>() }.check().is_err()
}
unsafe extern "C" fn progress(
    _context: *mut whisper_context,
    _state: *mut whisper_state,
    percent: i32,
    pointer: *mut c_void,
) {
    let observer = unsafe { &mut *pointer.cast::<Observer>() };
    let percent = percent.clamp(0, 99);
    if observer.last == percent || observer.error.is_some() {
        return;
    }
    observer.last = percent;
    observer.seq += 1;
    let publication = observer.output.notification(
        "speech.progress",
        &SpeechProgressEvent {
            request_id: observer.identity.request_id.clone(),
            attempt_epoch: observer.identity.attempt_epoch,
            seq: observer.seq,
            event_type: SpeechProgressEventType::Progress,
            stage: "transcribing".into(),
            progress: percent as u8,
            language: None,
            partial: None,
        },
    );
    if let Err(error) = publication {
        observer.error = Some(error);
        observer.cancel.cancel();
    }
}
