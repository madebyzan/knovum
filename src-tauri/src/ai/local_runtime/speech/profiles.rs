use crate::ai::contracts::speech_wire::AsrEngine;
use crate::local_resources::model_catalog::LocalModelId;

pub(crate) struct TranscriptionProfile {
    pub engine: AsrEngine,
    pub device: &'static str,
    pub compute_type: &'static str,
    pub id: &'static str,
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub(crate) fn transcription_profile(model: LocalModelId) -> Result<TranscriptionProfile, String> {
    Ok(TranscriptionProfile {
        engine: AsrEngine::MlxWhisper,
        device: "mps",
        compute_type: "float16",
        id: match model {
            LocalModelId::WhisperSmall => "mlx-whisper-small-mps-v1",
            LocalModelId::WhisperTiny => "mlx-whisper-tiny-mps-v1",
            LocalModelId::WhisperLargeV3 => "mlx-whisper-large-v3-mps-v1",
            _ => return Err("Selected model cannot transcribe audio".into()),
        },
    })
}

#[cfg(all(
    any(target_os = "macos", target_os = "windows"),
    target_arch = "x86_64"
))]
pub(crate) fn transcription_profile(model: LocalModelId) -> Result<TranscriptionProfile, String> {
    Ok(TranscriptionProfile {
        engine: AsrEngine::FasterWhisper,
        device: "cpu",
        compute_type: "int8",
        id: match model {
            LocalModelId::WhisperSmall => "faster-whisper-small-cpu-int8-v1",
            LocalModelId::WhisperTiny => "faster-whisper-tiny-cpu-int8-v1",
            LocalModelId::WhisperLargeV3 => "faster-whisper-large-v3-cpu-int8-v1",
            _ => return Err("Selected model cannot transcribe audio".into()),
        },
    })
}

#[cfg(not(knovum_supported_desktop))]
pub(crate) fn transcription_profile(_model: LocalModelId) -> Result<TranscriptionProfile, String> {
    Err("unsupported_device: ASR requires macOS arm64/x64 or Windows x64".into())
}

#[cfg(test)]
#[path = "tests/profiles.rs"]
mod tests;

pub(crate) async fn transcription_execution_profile<R: tauri::Runtime>(
    _app: &tauri::AppHandle<R>,
    model: LocalModelId,
    _cancellation: &tokio_util::sync::CancellationToken,
    _deadline_ms: u64,
) -> Result<TranscriptionProfile, String> {
    let profile = transcription_profile(model)?;
    #[cfg(windows)]
    {
        use crate::{
            ai::contracts::runtime::{RuntimeCapability, RuntimeDeviceProfile},
            ai::local_runtime::device_profiles::{backend_probe, memory_requirement},
            local_resources::runtimes::python::{self, RuntimeProfile},
        };
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Backends {
            cuda: bool,
            mps: bool,
        }
        let peak = memory_requirement(
            RuntimeCapability::SpeechTranscription,
            &RuntimeDeviceProfile {
                device: profile.device.into(),
                execution_profile_id: profile.id.into(),
                cpu_threads: 1,
            },
        )?
        .peak_bytes();
        let app = _app.clone();
        let runtime = crate::blocking::run(move || {
            python::resolve(&app, RuntimeProfile::TorchInference)?.ok_or_else(|| {
                "runtime_mismatch: prepared transcription runtime is missing".to_owned()
            })
        })
        .await?;
        let backends: Backends = backend_probe::report(
            &runtime,
            backend_probe::Probe::Transcription(peak),
            _cancellation,
            _cancellation,
            _deadline_ms,
        )
        .await?;
        if backends.mps {
            return Err("runtime_mismatch: Windows ASR cannot select Metal".into());
        }
        if backends.cuda {
            return faster_cuda_profile(model);
        }
    }
    Ok(profile)
}

#[cfg(any(windows, test))]
fn faster_cuda_profile(model: LocalModelId) -> Result<TranscriptionProfile, String> {
    Ok(TranscriptionProfile {
        engine: AsrEngine::FasterWhisper,
        device: "cuda",
        compute_type: "float16",
        id: match model {
            LocalModelId::WhisperSmall => "faster-whisper-small-cuda-f16-v1",
            LocalModelId::WhisperTiny => "faster-whisper-tiny-cuda-f16-v1",
            LocalModelId::WhisperLargeV3 => "faster-whisper-large-v3-cuda-f16-v1",
            _ => return Err("Selected model cannot transcribe audio".into()),
        },
    })
}
