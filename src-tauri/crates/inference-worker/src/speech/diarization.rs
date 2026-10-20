use super::{audio, ffi};
use crate::{cancellation::Cancellation, initialization::VerifiedRuntime};
use ffi::sherpa::*;
use knovum_inference_contracts::{
    runtime::RuntimeInitializeParams,
    speech_wire::{SpeakerTurn, SpeechDiarization, SpeechDiarizeParams},
};
use std::{path::PathBuf, ptr};

pub(crate) struct Diarizer(Engine);
struct Engine {
    api: SherpaApi,
    handle: *const SherpaOnnxOfflineSpeakerDiarization,
}
impl Drop for Engine {
    fn drop(&mut self) {
        unsafe {
            self.api
                .SherpaOnnxDestroyOfflineSpeakerDiarization(self.handle);
        }
    }
}

/// Admits a diarization profile and returns the snapshot it names. Pure in its initialization
/// message, so a precision that differs from the frozen exports is rejected before a graph loads.
pub(super) fn admit(params: &RuntimeInitializeParams) -> Result<&str, String> {
    if params.device_profile.device != "cpu"
        || params.device_profile.execution_profile_id != params.capability_spec_id
        || !(1..=8).contains(&params.device_profile.cpu_threads)
        || !matches!(
            params.capability_spec_id.as_str(),
            "sherpa-diarization-cpu-v1" | "sherpa-diarization-fp32-cpu-v1"
        )
    {
        return Err("unsupported_device: diarization profile is unavailable".into());
    }
    params
        .model_paths
        .speech_diarization
        .as_deref()
        .ok_or_else(|| "model_missing: diarization model is missing".into())
}

impl Diarizer {
    pub(crate) fn new(
        params: &RuntimeInitializeParams,
        runtime: &VerifiedRuntime,
    ) -> Result<Self, String> {
        let directory = PathBuf::from(admit(params)?);
        let segment = ffi::string(
            directory
                .join("segmentation.onnx")
                .to_str()
                .ok_or("model_corrupt: model path is invalid")?,
        )?;
        let embedding = ffi::string(
            directory
                .join("embedding.onnx")
                .to_str()
                .ok_or("model_corrupt: model path is invalid")?,
        )?;
        let api = ffi::sherpa_api(runtime)?;
        let mut config = configuration(-1);
        config.segmentation.pyannote.model = segment.as_ptr();
        config.embedding.model = embedding.as_ptr();
        config.segmentation.num_threads = i32::from(params.device_profile.cpu_threads);
        config.embedding.num_threads = config.segmentation.num_threads;
        let handle = unsafe { api.SherpaOnnxCreateOfflineSpeakerDiarization(&config) };
        if handle.is_null() {
            return Err("model_corrupt: diarization engine could not load".into());
        }
        Ok(Self(Engine { api, handle }))
    }

    pub(crate) fn diarize(
        &mut self,
        params: SpeechDiarizeParams,
        cancel: &Cancellation,
    ) -> Result<SpeechDiarization, String> {
        let bounds = &params.speaker_count_bounds;
        if bounds.minimum != bounds.maximum
            || bounds
                .minimum
                .is_some_and(|count| !(1..=32).contains(&count))
        {
            return Err(
                "invalid_input: specify one to 32 known speakers or automatic detection".into(),
            );
        }
        let audio = audio::read(&params.audio_ref, 16_000, 1, 16_000 * 7200, cancel)?;
        let engine = &self.0;
        cancel.check()?;
        let config = configuration(bounds.minimum.map_or(-1, i32::from));
        unsafe {
            engine
                .api
                .SherpaOnnxOfflineSpeakerDiarizationSetConfig(engine.handle, &config);
        }
        // This upstream callback cannot interrupt inference. The host's five-second kill owns hard cancellation.
        let diarization = unsafe {
            engine
                .api
                .SherpaOnnxOfflineSpeakerDiarizationProcessWithCallback(
                    engine.handle,
                    audio.samples.as_ptr(),
                    audio.samples.len() as i32,
                    Some(observe),
                    std::ptr::from_ref(cancel).cast_mut().cast(),
                )
        };
        if diarization.is_null() {
            cancel.check()?;
            return Err("diarization_failed: inference engine returned no result".into());
        }
        let outcome = (|| {
            cancel.check()?;
            let count = unsafe {
                engine
                    .api
                    .SherpaOnnxOfflineSpeakerDiarizationResultGetNumSegments(diarization)
            };
            if !(0..=100_000).contains(&count) {
                return Err("invalid_output: diarization returned excessive segments".into());
            }
            let mut turns = Vec::with_capacity(count as usize);
            if count > 0 {
                let segments = unsafe {
                    engine
                        .api
                        .SherpaOnnxOfflineSpeakerDiarizationResultSortByStartTime(diarization)
                };
                if segments.is_null() {
                    return Err("resource_exhausted: diarization segment allocation failed".into());
                }
                let converted: Result<(), String> = (|| {
                    for segment in unsafe { std::slice::from_raw_parts(segments, count as usize) } {
                        let (start, end) = (segment.start, segment.end);
                        if !start.is_finite()
                            || !end.is_finite()
                            || start < 0.0
                            || end <= start
                            || f64::from(end) > audio.frames() as f64 / 16_000.0 + 0.001
                            || segment.speaker < 0
                            || segment.speaker >= 32
                        {
                            return Err(
                                "invalid_output: diarization returned an invalid speaker timeline"
                                    .into(),
                            );
                        }
                        turns.push(SpeakerTurn {
                            speaker_id: segment.speaker.to_string(),
                            start_ms: (f64::from(start) * 1000.0).round() as u64,
                            end_ms: (f64::from(end) * 1000.0).round() as u64,
                        });
                    }
                    Ok(())
                })();
                unsafe {
                    engine
                        .api
                        .SherpaOnnxOfflineSpeakerDiarizationDestroySegment(segments);
                }
                converted?;
            }
            cancel.check()?;
            Ok(SpeechDiarization {
                request_id: params.identity.request_id,
                attempt_epoch: params.identity.attempt_epoch,
                model_spec_id: params.identity.model_spec_id,
                turns,
            })
        })();
        unsafe {
            engine
                .api
                .SherpaOnnxOfflineSpeakerDiarizationDestroyResult(diarization);
        }
        outcome
    }
}
fn configuration(clusters: i32) -> SherpaOnnxOfflineSpeakerDiarizationConfig {
    SherpaOnnxOfflineSpeakerDiarizationConfig {
        segmentation: SherpaOnnxOfflineSpeakerSegmentationModelConfig {
            pyannote: SherpaOnnxOfflineSpeakerSegmentationPyannoteModelConfig {
                model: ptr::null(),
                window_shift_ratio: 0.1,
            },
            num_threads: 1,
            debug: 0,
            provider: c"cpu".as_ptr(),
        },
        embedding: SherpaOnnxSpeakerEmbeddingExtractorConfig {
            model: ptr::null(),
            num_threads: 1,
            debug: 0,
            provider: c"cpu".as_ptr(),
        },
        clustering: SherpaOnnxFastClusteringConfig {
            num_clusters: clusters,
            threshold: 0.5,
            compute_confidence: 0,
        },
        min_duration_on: 0.3,
        min_duration_off: 0.5,
    }
}
unsafe extern "C" fn observe(_done: i32, _total: i32, pointer: *mut std::ffi::c_void) -> i32 {
    i32::from(unsafe { &*pointer.cast::<Cancellation>() }.check().is_err())
}
