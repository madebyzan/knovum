use super::{audio, native};
use crate::cancellation::Cancellation;
use knovum_inference_contracts::{runtime::RuntimeInitializeParams, speech_wire::*};
use native::audio::*;
use std::{ffi::CStr, path::PathBuf, ptr};

pub(crate) struct Aligner {
    params: RuntimeInitializeParams,
    engine: Option<Engine>,
}
struct Engine {
    api: AudioApi,
    session: *mut audiocpp_session,
}
impl Drop for Engine {
    fn drop(&mut self) {
        unsafe {
            self.api.audiocpp_session_free(self.session);
        }
    }
}
impl Engine {
    fn check(&self, status: u32) -> Result<(), String> {
        check(&self.api, status)
    }
    fn words(
        &self,
        samples: &[f32],
        text: &str,
        language: &str,
    ) -> Result<Vec<(String, u64, u64)>, String> {
        let request = unsafe { self.api.audiocpp_request_create() };
        if request.is_null() {
            return Err("resource_exhausted: alignment request allocation failed".into());
        }
        let mut result = ptr::null_mut();
        let outcome = (|| {
            let text_c = native::string(text)?;
            let language = native::string(language)?;
            self.check(unsafe {
                self.api
                    .audiocpp_request_set_text(request, text_c.as_ptr(), ptr::null())
            })?;
            self.check(unsafe {
                self.api
                    .audiocpp_request_set_text_language(request, language.as_ptr())
            })?;
            self.check(unsafe {
                self.api.audiocpp_request_set_audio(
                    request,
                    samples.as_ptr(),
                    samples.len(),
                    16_000,
                    1,
                )
            })?;
            self.check(unsafe {
                self.api
                    .audiocpp_session_run(self.session, request, &mut result)
            })?;
            if result.is_null() {
                return Err("invalid_output: alignment returned no result".into());
            }
            let count = unsafe { self.api.audiocpp_result_word_count(result) };
            if count > text.chars().count() {
                return Err("invalid_output: alignment exceeds its transcript length".into());
            }
            let mut words = Vec::with_capacity(count);
            for index in 0..count {
                let mut word = ptr::null();
                let (mut start, mut end) = (0i64, 0i64);
                self.check(unsafe {
                    self.api.audiocpp_result_word(
                        result,
                        index,
                        &mut word,
                        &mut start,
                        &mut end,
                        ptr::null_mut(),
                    )
                })?;
                if word.is_null() || start < 0 || end <= start || end as usize > samples.len() {
                    return Err(
                        "invalid_output: alignment returned an invalid word timeline".into(),
                    );
                }
                let word = unsafe { CStr::from_ptr(word) }
                    .to_str()
                    .map_err(|error| format!("invalid_output: {error}"))?
                    .to_owned();
                if word.is_empty() {
                    return Err("invalid_output: alignment returned an empty word".into());
                }
                words.push((
                    word,
                    (start as u64 * 1000).div_ceil(16_000),
                    (end as u64 * 1000).div_ceil(16_000),
                ));
            }
            Ok(words)
        })();
        unsafe {
            self.api.audiocpp_result_free(result);
            self.api.audiocpp_request_free(request);
        }
        outcome
    }
}
impl Aligner {
    pub(crate) fn new(params: &RuntimeInitializeParams) -> Result<Self, String> {
        if alignment_execution_id(&params.capability_spec_id, &params.device_profile.device)
            .as_deref()
            != Some(params.device_profile.execution_profile_id.as_str())
            || !(1..=8).contains(&params.device_profile.cpu_threads)
            || params.model_paths.speech_alignment.is_none()
        {
            return Err("unsupported_device: alignment profile is unavailable".into());
        }
        Ok(Self {
            params: params.clone(),
            engine: None,
        })
    }
    fn load(&mut self) -> Result<&Engine, String> {
        if self.engine.is_none() {
            let api = unsafe {
                AudioApi::from_library(native::load(&native::path(&self.params, "audiocpp")?)?)
            }
            .map_err(|error| format!("runtime_mismatch: {error}"))?;
            if unsafe { api.audiocpp_abi_version() } != 0x000200 {
                return Err("runtime_mismatch: alignment ABI differs".into());
            }
            let directory = PathBuf::from(
                self.params
                    .model_paths
                    .speech_alignment
                    .as_ref()
                    .ok_or("model_missing: alignment model is missing")?,
            );
            let model_path =
                if self.params.capability_spec_id == "qwen3-forced-aligner-0.6b-f16-native-v2" {
                    directory.join("qwen3-forced-aligner-0.6b-f16.gguf")
                } else {
                    directory.join("qwen3-forced-aligner-0.6b-q8_0.gguf")
                };
            let path = native::string(
                model_path
                    .to_str()
                    .ok_or("model_corrupt: alignment path is invalid")?,
            )?;
            let mut registry = ptr::null_mut();
            let mut model = ptr::null_mut();
            let mut session = ptr::null_mut();
            check(&api, unsafe {
                api.audiocpp_registry_create(ptr::null(), &mut registry)
            })?;
            let result = (|| {
                let config = audiocpp_model_config {
                    family_hint: c"qwen3_forced_aligner".as_ptr(),
                    config_id: ptr::null(),
                    weight_id: ptr::null(),
                    model_spec_override: ptr::null(),
                };
                check(&api, unsafe {
                    api.audiocpp_model_load(
                        registry,
                        path.as_ptr(),
                        &config,
                        ptr::null(),
                        &mut model,
                    )
                })?;
                let backend = match self.params.device_profile.device.as_str() {
                    "cpu" => c"cpu",
                    "mps" => c"metal",
                    "vulkan" => c"vulkan",
                    _ => return Err("unsupported_device: alignment backend is invalid".into()),
                };
                let config = audiocpp_backend_config {
                    backend: backend.as_ptr(),
                    device: 0,
                    threads: i32::from(self.params.device_profile.cpu_threads),
                };
                check(&api, unsafe {
                    api.audiocpp_session_create(
                        model,
                        c"align".as_ptr(),
                        c"offline".as_ptr(),
                        &config,
                        ptr::null(),
                        &mut session,
                    )
                })
            })();
            unsafe {
                api.audiocpp_model_free(model);
                api.audiocpp_registry_free(registry);
            }
            if let Err(error) = result {
                unsafe {
                    api.audiocpp_session_free(session);
                }
                return Err(error);
            }
            if session.is_null() {
                return Err("model_corrupt: alignment session is absent".into());
            }
            self.engine = Some(Engine { api, session });
        }
        self.engine
            .as_ref()
            .ok_or("worker_crashed: alignment engine is unavailable".into())
    }
    pub(crate) fn align(
        &mut self,
        params: SpeechAlignParams,
        cancel: &Cancellation,
    ) -> Result<SpeechAlignment, String> {
        if params.profile.device != self.params.device_profile.device
            || !(1..=8).contains(&params.text_segments.len())
        {
            return Err("runtime_mismatch: alignment request differs from its session".into());
        }
        let language = match params.profile.language.as_str() {
            "zh" => "Chinese",
            "en" => "English",
            "yue" => "Cantonese",
            "fr" => "French",
            "de" => "German",
            "it" => "Italian",
            "pt" => "Portuguese",
            "ru" => "Russian",
            "es" => "Spanish",
            _ => return Err("unsupported_language: alignment language is unsupported".into()),
        };
        let audio = audio::read(&params.audio_ref, 16_000, 1, 16_000 * 7200, cancel)?;
        for segment in &params.text_segments {
            if segment.text.trim().is_empty()
                || segment.text.len() > 65_536
                || segment.end_ms <= segment.start_ms
                || segment.end_ms - segment.start_ms > 300_000
                || segment.end_ms > audio.frames() as u64 / 16
            {
                return Err("invalid_input: alignment segment exceeds its source timeline".into());
            }
        }
        let engine = self.load()?;
        cancel.check()?;
        let mut units = Vec::new();
        for segment in params.text_segments {
            let start = segment.start_ms as usize * 16;
            let end = segment.end_ms as usize * 16;
            let words = engine.words(&audio.samples[start..end], &segment.text, language)?;
            cancel.check()?;
            let kind = if matches!(language, "Chinese" | "Cantonese") {
                AlignmentUnitKind::Character
            } else {
                AlignmentUnitKind::Word
            };
            if words.is_empty() {
                units.push(AlignedSpeechUnit {
                    segment_id: segment.segment_id,
                    text: segment.text,
                    unit_kind: kind,
                    start_ms: None,
                    end_ms: None,
                    score: None,
                    unaligned_reason: Some("no_alignment".into()),
                });
                continue;
            }
            for (word, start, end) in words {
                units.push(AlignedSpeechUnit {
                    segment_id: segment.segment_id.clone(),
                    text: word,
                    unit_kind: kind.clone(),
                    start_ms: Some(segment.start_ms + start),
                    end_ms: Some(segment.start_ms + end),
                    score: None,
                    unaligned_reason: None,
                });
            }
        }
        let coverage =
            units.iter().filter(|unit| unit.start_ms.is_some()).count() as f32 / units.len() as f32;
        Ok(SpeechAlignment {
            request_id: params.identity.request_id,
            attempt_epoch: params.identity.attempt_epoch,
            model_spec_id: params.identity.model_spec_id,
            units,
            coverage,
        })
    }
}
fn check(api: &AudioApi, status: u32) -> Result<(), String> {
    if status == 0 {
        return Ok(());
    }
    let error = unsafe { api.audiocpp_last_error() };
    let message = if error.is_null() {
        "native alignment failed".into()
    } else {
        unsafe { CStr::from_ptr(error) }.to_string_lossy()
    };
    Err(format!("alignment_failed: {message}"))
}
