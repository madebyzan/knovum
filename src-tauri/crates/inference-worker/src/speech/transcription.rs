use super::{artifacts, audio, ffi};
use crate::{artifact, cancellation::Cancellation, protocol::Output};
use ffi::whisper::*;
use knovum_inference_contracts::{
    execution_profile::{self, WHISPER_VARIANTS},
    runtime::RuntimeInitializeParams,
    speech_wire::{
        AsrEngine, LanguagePolicy, SpeechTranscribeParams, SpeechTranscriptArtifact,
        SpeechTranscription,
    },
};
use std::path::{Path, PathBuf};

pub(crate) struct Transcriber {
    params: RuntimeInitializeParams,
    engine: Option<Engine>,
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
impl Transcriber {
    pub(crate) fn new(params: &RuntimeInitializeParams) -> Result<Self, String> {
        let device = params.device_profile.device.as_str();
        let allowed = WHISPER_VARIANTS.into_iter().any(|(model, precision)| {
            params.capability_spec_id == execution_profile::whisper(model, device, precision)
        });
        if !allowed
            || params.device_profile.execution_profile_id != params.capability_spec_id
            || params.model_paths.speech_transcription.is_none()
            || !matches!(device, "cpu" | "mps")
            || device == "mps" && !cfg!(all(target_os = "macos", target_arch = "aarch64"))
            || !(1..=8).contains(&params.device_profile.cpu_threads)
        {
            return Err("unsupported_device: Whisper execution profile is unavailable".into());
        }
        Ok(Self {
            params: params.clone(),
            engine: None,
        })
    }
    fn load(&mut self) -> Result<&Engine, String> {
        if self.engine.is_none() {
            let directory = PathBuf::from(
                self.params
                    .model_paths
                    .speech_transcription
                    .as_ref()
                    .ok_or("model_missing: transcription model is missing")?,
            );
            let model = directory.join("model.bin");
            let file = artifact::open_regular(&model)?;
            if !(1..=3_100_000_000)
                .contains(&file.metadata().map_err(|error| error.to_string())?.len())
            {
                return Err("model_corrupt: Whisper weight size is invalid".into());
            }
            let library = ffi::path(&self.params, "whisper")?;
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
            require_device(&ggml, &self.params.device_profile.device)?;
            let api = unsafe { WhisperApi::from_library(ffi::load(&library)?) }
                .map_err(|error| format!("runtime_mismatch: {error}"))?;
            let path = ffi::string(
                model
                    .to_str()
                    .ok_or("model_corrupt: Whisper path is invalid")?,
            )?;
            let mut options = unsafe { api.whisper_context_default_params() };
            options.use_gpu = self.params.device_profile.device != "cpu";
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
            self.engine = Some(Engine {
                api,
                threads: self.params.device_profile.cpu_threads,
                _ggml: ggml,
                context,
                state,
            });
        }
        self.engine
            .as_ref()
            .ok_or("worker_crashed: Whisper engine is unavailable".into())
    }
    pub(crate) fn transcribe(
        &mut self,
        work: &Path,
        params: SpeechTranscribeParams,
        cancel: &Cancellation,
        output: &Output,
    ) -> Result<SpeechTranscription, String> {
        if !matches!(params.profile.engine, AsrEngine::WhisperCpp)
            || params.profile.device != self.params.device_profile.device
            || params.profile.compute_type
                != if self.params.capability_spec_id.contains("-f16-") {
                    "f16"
                } else {
                    "q8_0"
                }
            || !(1..=10).contains(&params.profile.beam_size)
        {
            return Err("runtime_mismatch: transcription profile differs".into());
        }
        let audio = audio::read(&params.audio_ref, 16_000, 1, 16_000 * 7200, cancel)?;
        let threads = self.params.device_profile.cpu_threads;
        let directory = PathBuf::from(
            self.params
                .model_paths
                .speech_transcription
                .as_ref()
                .ok_or("model_missing: transcription model is missing")?,
        );
        let vad = ffi::string(
            directory
                .join("vad.bin")
                .to_str()
                .ok_or("model_corrupt: VAD path is invalid")?,
        )?;
        let engine = self.load()?;
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
            let file = artifact::open_regular(&directory.join("vad.bin"))?;
            if !(1..=16 * 1024 * 1024)
                .contains(&file.metadata().map_err(|error| error.to_string())?.len())
            {
                return Err("model_corrupt: Silero VAD weight size is invalid".into());
            }
            super::whisper_vad::regions(&engine.api, &vad, &audio.samples, threads, cancel)?
        } else {
            std::iter::once(0..duration_ms).collect()
        };
        let (language, segments) = super::whisper_decode::decode(
            engine, &params, &audio, &language, &regions, cancel, output,
        )?;
        let speech_duration_ms = regions.iter().map(|region| region.end - region.start).sum();
        let transcript = SpeechTranscriptArtifact {
            language,
            duration_ms: audio.frames() as u64 / 16,
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
