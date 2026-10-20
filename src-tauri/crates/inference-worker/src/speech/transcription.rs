use super::{artifacts, audio, ffi};
use crate::{
    artifact, cancellation::Cancellation, initialization::VerifiedRuntime, protocol::Output,
};
use ffi::whisper::*;
use knovum_inference_contracts::{
    execution_profile::{self, WHISPER_VARIANTS},
    runtime::RuntimeInitializeParams,
    speech_wire::{
        AsrEngine, LanguagePolicy, SpeechTranscribeParams, SpeechTranscriptArtifact,
        SpeechTranscription,
    },
};
use std::{
    ffi::CString,
    path::{Path, PathBuf},
};

pub(crate) struct Transcriber {
    engine: Engine,
    device: String,
    compute_type: &'static str,
    vad: CString,
}
pub(super) struct Engine {
    pub(super) api: WhisperApi,
    pub(super) threads: u8,
    _ggml: GgmlApi,
    pub(super) context: *mut whisper_context,
    pub(super) state: *mut whisper_state,
}
impl Drop for Engine {
    fn drop(&mut self) {
        unsafe {
            self.api.whisper_free_state(self.state);
            self.api.whisper_free(self.context);
        }
    }
}

/// Admits a Whisper execution profile and returns the snapshot it names. Pure in its initialization
/// message, so an obsolete profile is rejected before a library or a weight is read.
pub(super) fn admit(params: &RuntimeInitializeParams) -> Result<&str, String> {
    let device = params.device_profile.device.as_str();
    let allowed = WHISPER_VARIANTS.into_iter().any(|(model, precision)| {
        params.capability_spec_id == execution_profile::whisper(model, device, precision)
    });
    if !allowed
        || params.device_profile.execution_profile_id != params.capability_spec_id
        || !matches!(device, "cpu" | "mps")
        || device == "mps" && !cfg!(all(target_os = "macos", target_arch = "aarch64"))
        || !(1..=8).contains(&params.device_profile.cpu_threads)
    {
        return Err("unsupported_device: Whisper execution profile is unavailable".into());
    }
    params
        .model_paths
        .speech_transcription
        .as_deref()
        .ok_or_else(|| "model_missing: transcription model is missing".into())
}

impl Transcriber {
    pub(crate) fn new(
        params: &RuntimeInitializeParams,
        runtime: &VerifiedRuntime,
    ) -> Result<Self, String> {
        let directory = PathBuf::from(admit(params)?);
        let model = directory.join("model.bin");
        require_bounded_weight(&model, 3_100_000_000, "Whisper")?;
        let vad_model = directory.join("vad.bin");
        require_bounded_weight(&vad_model, 16 * 1024 * 1024, "Silero VAD")?;
        let vad = ffi::string(
            vad_model
                .to_str()
                .ok_or("model_corrupt: VAD path is invalid")?,
        )?;
        let library = ffi::library(runtime, "whisper")?;
        let ggml_library = library.with_file_name(if cfg!(windows) {
            "ggml.dll"
        } else {
            "libggml.0.dylib"
        });
        let ggml = unsafe { GgmlApi::from_library(ffi::load(&ggml_library)?) }
            .map_err(|error| format!("runtime_mismatch: {error}"))?;
        let backends = ffi::string(
            library
                .parent()
                .and_then(Path::to_str)
                .ok_or("runtime_mismatch: Whisper backend directory is invalid")?,
        )?;
        // Upstream selects the best supported CPU variant only from the sealed supplier directory.
        unsafe { ggml.ggml_backend_load_all_from_path(backends.as_ptr()) };
        require_device(&ggml, &params.device_profile.device)?;
        let api = unsafe { WhisperApi::from_library(ffi::load(&library)?) }
            .map_err(|error| format!("runtime_mismatch: {error}"))?;
        let path = ffi::string(
            model
                .to_str()
                .ok_or("model_corrupt: Whisper path is invalid")?,
        )?;
        let mut options = unsafe { api.whisper_context_default_params() };
        options.use_gpu = params.device_profile.device != "cpu";
        // Upstream's file loader converts UTF-8 to UTF-16 on Windows; no path aliases are required.
        let context =
            unsafe { api.whisper_init_from_file_with_params_no_state(path.as_ptr(), options) };
        if context.is_null() {
            return Err("model_corrupt: Whisper could not load".into());
        }
        let state = unsafe { api.whisper_init_state(context) };
        if state.is_null() {
            unsafe {
                api.whisper_free(context);
            }
            return Err("resource_exhausted: Whisper state allocation failed".into());
        }
        Ok(Self {
            engine: Engine {
                api,
                threads: params.device_profile.cpu_threads,
                _ggml: ggml,
                context,
                state,
            },
            device: params.device_profile.device.clone(),
            compute_type: if params.capability_spec_id.contains("-f16-") {
                "f16"
            } else {
                "q8_0"
            },
            vad,
        })
    }

    pub(crate) fn transcribe(
        &mut self,
        work: &Path,
        params: SpeechTranscribeParams,
        cancel: &Cancellation,
        output: &Output,
    ) -> Result<SpeechTranscription, String> {
        if !matches!(params.profile.engine, AsrEngine::WhisperCpp)
            || params.profile.device != self.device
            || params.profile.compute_type != self.compute_type
            || !(1..=10).contains(&params.profile.beam_size)
        {
            return Err("runtime_mismatch: transcription profile differs".into());
        }
        let audio = audio::read(&params.audio_ref, 16_000, 1, 16_000 * 7200, cancel)?;
        let engine = &self.engine;
        cancel.check()?;
        let language = match params.profile.language_policy {
            LanguagePolicy::Detect => ffi::string("auto")?,
            LanguagePolicy::Fixed => {
                let selected = ffi::string(
                    params
                        .profile
                        .language
                        .as_deref()
                        .ok_or("invalid_input: fixed language is missing")?,
                )?;
                if unsafe { engine.api.whisper_lang_id(selected.as_ptr()) } < 0 {
                    return Err("unsupported_language: Whisper language is unsupported".into());
                }
                selected
            }
        };
        let duration_ms = audio.frames() as u64 / 16;
        let regions = if params.profile.vad_enabled {
            super::whisper_vad::regions(
                &engine.api,
                &self.vad,
                &audio.samples,
                engine.threads,
                cancel,
            )?
        } else {
            std::iter::once(0..duration_ms).collect()
        };
        let (language, segments) = super::whisper_decode::decode(
            engine, &params, &audio, &language, &regions, cancel, output,
        )?;
        let speech_duration_ms = regions.iter().map(|region| region.end - region.start).sum();
        let transcript = SpeechTranscriptArtifact {
            language,
            duration_ms,
            speech_duration_ms,
            segments,
        };
        let reference = artifacts::json(work, &params.identity.request_id, &transcript)?;
        Ok(SpeechTranscription {
            request_id: params.identity.request_id,
            attempt_epoch: params.identity.attempt_epoch,
            model_spec_id: params.identity.model_spec_id,
            transcript_ref: reference,
        })
    }
}

/// Establishes that a supplied weight is a regular file within its frozen size bound, once, while
/// the worker initializes rather than on every request that reads it.
fn require_bounded_weight(path: &Path, maximum: u64, subject: &str) -> Result<(), String> {
    let file = artifact::open_regular(path)?;
    if !(1..=maximum).contains(&file.metadata().map_err(|error| error.to_string())?.len()) {
        return Err(format!("model_corrupt: {subject} weight size is invalid"));
    }
    Ok(())
}

fn require_device(api: &GgmlApi, device: &str) -> Result<(), String> {
    let count = unsafe { api.ggml_backend_dev_count() };
    if !(1..=256).contains(&count) {
        return Err("runtime_mismatch: Whisper backend inventory is invalid".into());
    }
    for index in 0..count {
        let backend_device = unsafe { api.ggml_backend_dev_get(index) };
        if backend_device.is_null() {
            return Err("runtime_mismatch: Whisper returned an invalid device".into());
        }
        let kind = unsafe { api.ggml_backend_dev_type(backend_device) };
        if device == "cpu" && kind == ggml_backend_dev_type_GGML_BACKEND_DEVICE_TYPE_CPU
            || device == "mps"
                && (kind == ggml_backend_dev_type_GGML_BACKEND_DEVICE_TYPE_GPU
                    || kind == ggml_backend_dev_type_GGML_BACKEND_DEVICE_TYPE_IGPU)
        {
            return Ok(());
        }
    }
    Err("unsupported_device: selected Whisper device is unavailable".into())
}
