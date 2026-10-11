use super::{audio, ffi, stream::Publisher};
use crate::cancellation::Cancellation;
use ffi::voice::*;
use knovum_inference_contracts::{
    runtime::RuntimeInitializeParams,
    speech_wire::{SpeechSynthesizeParams, VoiceSourceKind},
};
use std::{
    ffi::{c_void, CStr},
    path::PathBuf,
    ptr,
};

pub(crate) struct VoxCpm {
    params: RuntimeInitializeParams,
    engine: Option<Engine>,
}
struct Engine {
    api: VoiceApi,
    handle: *mut c_void,
}
impl Drop for Engine {
    fn drop(&mut self) {
        unsafe {
            self.api.knovum_voice_close(self.handle);
        }
    }
}
impl VoxCpm {
    pub(super) fn new(params: &RuntimeInitializeParams) -> Result<Self, String> {
        let quantization = match params.capability_spec_id.as_str() {
            "voxcpm2-2b-gguf-orig-v1" => "orig",
            "voxcpm2-2b-gguf-q8_0-v1" => "q8_0",
            _ => return Err("runtime_mismatch: VoxCPM2 model specification differs".into()),
        };
        let device = params.device_profile.device.as_str();
        if !matches!(device, "cpu" | "mps" | "vulkan")
            || !(1..=8).contains(&params.device_profile.cpu_threads)
            || params.model_paths.speech_synthesis.is_none()
            || params.device_profile.execution_profile_id
                != format!("voxcpm2-{device}-gguf-{quantization}-v1")
        {
            return Err("runtime_mismatch: VoxCPM2 model specification differs".into());
        }
        Ok(Self {
            params: params.clone(),
            engine: None,
        })
    }
    fn load(&mut self) -> Result<&Engine, String> {
        if self.engine.is_none() {
            let api = unsafe {
                VoiceApi::from_library(ffi::load(&ffi::path(&self.params, "knovum_voice")?)?)
            }
            .map_err(|error| format!("runtime_mismatch: {error}"))?;
            if unsafe { api.knovum_voice_abi() } != 3 {
                return Err("runtime_mismatch: VoxCPM2 bridge ABI differs".into());
            }
            let device = match self.params.device_profile.device.as_str() {
                "cpu" => 0,
                "mps" => 1,
                "vulkan" => 3,
                _ => return Err("unsupported_device: VoxCPM2 backend is unavailable".into()),
            };
            let devices = unsafe { api.knovum_audio_devices(8 * 1024 * 1024 * 1024) };
            if devices < 0 {
                return Err(failure(&api, "runtime_mismatch"));
            }
            if devices & (1 << device) == 0 {
                return Err("unsupported_device: VoxCPM2 backend capacity is unavailable".into());
            }
            let directory = PathBuf::from(
                self.params
                    .model_paths
                    .speech_synthesis
                    .as_ref()
                    .ok_or("model_missing: synthesis model is missing")?,
            );
            let file = if self.params.capability_spec_id == "voxcpm2-2b-gguf-orig-v1" {
                "voxcpm2-orig.gguf"
            } else {
                "voxcpm2-q8_0.gguf"
            };
            let path = ffi::string(
                directory
                    .join(file)
                    .to_str()
                    .ok_or("model_corrupt: VoxCPM2 path is invalid")?,
            )?;
            let handle = unsafe {
                api.knovum_voice_open(
                    path.as_ptr(),
                    device,
                    i32::from(self.params.device_profile.cpu_threads),
                )
            };
            if handle.is_null() {
                return Err(failure(&api, "model_corrupt"));
            }
            self.engine = Some(Engine { api, handle });
        }
        self.engine
            .as_ref()
            .ok_or("worker_crashed: VoxCPM2 engine is unavailable".into())
    }
    pub(super) fn generate(
        &mut self,
        params: &SpeechSynthesizeParams,
        publisher: &mut Publisher,
        cancel: &Cancellation,
    ) -> Result<(), String> {
        let (text, reference, transcript) = inputs(params, cancel)?;
        let text = ffi::string(&text)?;
        let transcript = transcript.as_deref().map(ffi::string).transpose()?;
        let engine = self.load()?;
        cancel.check()?;
        let mut observer = Observer {
            publisher,
            error: None,
        };
        let samples = reference.as_deref().unwrap_or(&[]);
        let status = unsafe {
            engine.api.knovum_voice_generate(
                engine.handle,
                text.as_ptr(),
                if samples.is_empty() {
                    ptr::null()
                } else {
                    samples.as_ptr()
                },
                samples.len(),
                transcript
                    .as_ref()
                    .map_or(ptr::null(), |text| text.as_ptr()),
                params.seed as u32,
                4096,
                Some(publish),
                std::ptr::from_mut(&mut observer).cast(),
            )
        };
        if let Some(error) = observer.error {
            return Err(error);
        }
        cancel.check()?;
        if status != 0 {
            return Err(failure(&engine.api, "synthesis_failed"));
        }
        Ok(())
    }
}
type Conditioning = (String, Option<Vec<f32>>, Option<String>);
fn inputs(params: &SpeechSynthesizeParams, cancel: &Cancellation) -> Result<Conditioning, String> {
    let voice = &params.voice_revision;
    if voice.speaker_id.is_some() {
        return Err("invalid_input: VoxCPM2 does not accept preset speaker IDs".into());
    }
    match voice.source_kind {
        VoiceSourceKind::Designed => {
            let prompt = voice
                .design_prompt
                .as_deref()
                .filter(|text| !text.trim().is_empty())
                .ok_or("invalid_input: voice design prompt is missing")?;
            if voice.reference_audio.is_some()
                || voice.reference_text.is_some()
                || params.style.is_some()
            {
                return Err("invalid_input: voice design controls are inconsistent".into());
            }
            Ok((format!("({}){}", prompt.trim(), params.text), None, None))
        }
        VoiceSourceKind::Reference => {
            let reference = voice
                .reference_audio
                .as_ref()
                .ok_or("invalid_input: immutable reference audio is missing")?;
            let audio = audio::read(reference, 16_000, 1, 30 * 16_000, cancel)?;
            if audio.frames() < 5 * 16_000 {
                return Err(
                    "invalid_input: voice reference must last five to thirty seconds".into(),
                );
            }
            if let Some(text) = &voice.reference_text {
                if text.trim().is_empty() || voice.design_prompt.is_some() || params.style.is_some()
                {
                    return Err(
                        "invalid_input: transcript cloning controls are inconsistent".into(),
                    );
                }
                Ok((params.text.clone(), Some(audio.samples), Some(text.clone())))
            } else {
                let controls = [voice.design_prompt.as_deref(), params.style.as_deref()]
                    .into_iter()
                    .flatten()
                    .map(str::trim)
                    .filter(|text| !text.is_empty())
                    .collect::<Vec<_>>();
                let text = if controls.is_empty() {
                    params.text.clone()
                } else {
                    format!("({}){}", controls.join(", "), params.text)
                };
                Ok((text, Some(audio.samples), None))
            }
        }
        VoiceSourceKind::Preset => {
            Err("invalid_input: VoxCPM2 requires voice design or reference cloning".into())
        }
    }
}
struct Observer<'a> {
    publisher: &'a mut Publisher,
    error: Option<String>,
}
unsafe extern "C" fn publish(
    context: *mut c_void,
    pointer: *const f32,
    count: usize,
    rate: i32,
) -> i32 {
    let observer = unsafe { &mut *context.cast::<Observer<'_>>() };
    let synthesis = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if pointer.is_null() || count == 0 || count > 16 * 1024 || rate != 48_000 {
            return Err("invalid_output: VoxCPM2 chunk format differs".into());
        }
        observer.publisher.publish(
            unsafe { std::slice::from_raw_parts(pointer, count) },
            rate as u32,
        )
    }));
    match synthesis {
        Ok(Ok(())) => 0,
        Ok(Err(error)) => {
            observer.error = Some(error);
            1
        }
        Err(_) => {
            observer.error = Some("worker_crashed: voice callback panicked".into());
            1
        }
    }
}
fn failure(api: &VoiceApi, code: &str) -> String {
    let error = unsafe { api.knovum_voice_error() };
    let message = if error.is_null() {
        "VoxCPM2 failed".into()
    } else {
        unsafe { CStr::from_ptr(error) }.to_string_lossy()
    };
    format!("{code}: {message}")
}
