use super::{artifacts, audio, ffi};
use crate::{cancellation::Cancellation, initialization::VerifiedRuntime};
use ffi::sherpa::*;
use knovum_inference_contracts::{
    runtime::RuntimeInitializeParams,
    speech_wire::{SpeechSeparateParams, SpeechSeparation},
};
use std::{
    path::{Path, PathBuf},
    ptr,
};

pub(crate) struct Separator(Engine);
struct Engine {
    api: SherpaApi,
    handle: *const SherpaOnnxOfflineSourceSeparation,
}
impl Drop for Engine {
    fn drop(&mut self) {
        unsafe {
            self.api
                .SherpaOnnxDestroyOfflineSourceSeparation(self.handle);
        }
    }
}

/// Admits a source-separation profile and returns the snapshot it names. Pure in its initialization
/// message, so a precision that differs from the frozen pair is rejected before a graph loads.
pub(super) fn admit(params: &RuntimeInitializeParams) -> Result<&str, String> {
    if params.device_profile.device != "cpu"
        || params.device_profile.execution_profile_id != params.capability_spec_id
        || !(1..=8).contains(&params.device_profile.cpu_threads)
        || !matches!(
            params.capability_spec_id.as_str(),
            "spleeter-2stems-fp16-cpu-v1" | "spleeter-2stems-fp32-cpu-v1"
        )
    {
        return Err("unsupported_device: source separation profile is unavailable".into());
    }
    params.model.snapshot()
}

impl Separator {
    pub(crate) fn new(
        params: &RuntimeInitializeParams,
        runtime: &VerifiedRuntime,
    ) -> Result<Self, String> {
        let directory = PathBuf::from(admit(params)?);
        let half = params.capability_spec_id == "spleeter-2stems-fp16-cpu-v1";
        let name = |stem| {
            directory.join(if half {
                format!("{stem}.fp16.onnx")
            } else {
                format!("{stem}.onnx")
            })
        };
        let vocals = ffi::string(
            name("vocals")
                .to_str()
                .ok_or("model_corrupt: model path is invalid")?,
        )?;
        let accompaniment = ffi::string(
            name("accompaniment")
                .to_str()
                .ok_or("model_corrupt: model path is invalid")?,
        )?;
        let api = ffi::sherpa_api(runtime)?;
        let config = SherpaOnnxOfflineSourceSeparationConfig {
            model: SherpaOnnxOfflineSourceSeparationModelConfig {
                spleeter: SherpaOnnxOfflineSourceSeparationSpleeterModelConfig {
                    vocals: vocals.as_ptr(),
                    accompaniment: accompaniment.as_ptr(),
                },
                uvr: SherpaOnnxOfflineSourceSeparationUvrModelConfig { model: ptr::null() },
                num_threads: i32::from(params.device_profile.cpu_threads),
                debug: 0,
                provider: c"cpu".as_ptr(),
            },
        };
        let handle = unsafe { api.SherpaOnnxCreateOfflineSourceSeparation(&config) };
        if handle.is_null() {
            return Err("model_corrupt: source separation engine could not load".into());
        }
        Ok(Self(Engine { api, handle }))
    }

    pub(crate) fn separate(
        &mut self,
        work: &Path,
        params: SpeechSeparateParams,
        cancel: &Cancellation,
    ) -> Result<SpeechSeparation, String> {
        let input = audio::read(&params.audio_ref, 44_100, 2, 44_100 * 600, cancel)?;
        let frames = input.frames();
        let padded = frames.div_ceil(1024).max(4) * 1024;
        let mut left = vec![0.0f32; padded];
        let mut right = vec![0.0f32; padded];
        for (index, pair) in input.samples.chunks_exact(2).enumerate() {
            left[index] = pair[0];
            right[index] = pair[1];
        }
        let engine = &self.0;
        cancel.check()?;
        let channels = [left.as_ptr(), right.as_ptr()];
        let output = unsafe {
            engine.api.SherpaOnnxOfflineSourceSeparationProcess(
                engine.handle,
                channels.as_ptr(),
                2,
                padded as i32,
                44_100,
            )
        };
        if output.is_null() {
            cancel.check()?;
            return Err("separation_failed: inference engine returned no stems".into());
        }
        let separation = (|| {
            cancel.check()?;
            // The pinned engine owns this result and its two channel arrays until DestroyOutput.
            let output = unsafe { &*output };
            if output.num_stems != 2 || output.sample_rate != 44_100 || output.stems.is_null() {
                return Err("invalid_output: separation stem profile differs".into());
            }
            let stems = unsafe { std::slice::from_raw_parts(output.stems, 2) };
            let directory = artifacts::attempt(work, &params.identity.request_id)?;
            let mut refs = Vec::with_capacity(2);
            for (stem, name) in stems.iter().zip(["vocals.wav", "background.wav"]) {
                if stem.num_channels != 2 || stem.n != padded as i32 || stem.samples.is_null() {
                    return Err("invalid_output: separation stem timeline differs".into());
                }
                let pointers = unsafe { std::slice::from_raw_parts(stem.samples, 2) };
                if pointers.iter().any(|pointer| pointer.is_null()) {
                    return Err("invalid_output: separation channel is absent".into());
                }
                let channels = pointers
                    .iter()
                    .map(|pointer| unsafe { std::slice::from_raw_parts(*pointer, frames) })
                    .collect::<Vec<_>>();
                let mut samples = Vec::with_capacity(frames * 2);
                for (&left, &right) in channels[0].iter().zip(channels[1]) {
                    samples.extend([left, right]);
                }
                let mut reference = audio::write_wave(&directory, name, 44_100, 2, &samples)?;
                reference.file_name = format!("attempt-{}/{name}", params.identity.request_id);
                refs.push(reference);
                cancel.check()?;
            }
            Ok(SpeechSeparation {
                request_id: params.identity.request_id,
                attempt_epoch: params.identity.attempt_epoch,
                model_spec_id: params.identity.model_spec_id,
                vocals_ref: refs.remove(0),
                background_ref: refs.remove(0),
                sample_count: frames as u64,
            })
        })();
        unsafe {
            engine.api.SherpaOnnxDestroySourceSeparationOutput(output);
        }
        separation
    }
}
