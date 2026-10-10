use crate::ai::{
    contracts::runtime::{
        RuntimeCapability, RuntimeDeviceProfile, RuntimeInitializeParams, RuntimeModelPaths,
        PROTOCOL_VERSION,
    },
    local_runtime::process::Worker,
};
use crate::local_resources::runtimes::python;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, Runtime};
use tokio_util::sync::CancellationToken;

pub(super) struct WorkerStart<'a, R: Runtime> {
    pub(super) app: &'a AppHandle<R>,
    pub(super) runtime: &'a python::PythonRuntimePaths,
    pub(super) model_path: &'a Path,
    pub(super) capability: RuntimeCapability,
    pub(super) model_spec_id: String,
    pub(super) device_profile: RuntimeDeviceProfile,
    pub(super) deadline_ms: u64,
    pub(super) cancellation: &'a CancellationToken,
    pub(super) exit: &'a CancellationToken,
}

pub(super) async fn start_worker<R: Runtime>(
    start: WorkerStart<'_, R>,
    slot: &mut Option<Worker>,
) -> Result<PathBuf, String> {
    let WorkerStart {
        app,
        runtime,
        model_path,
        capability,
        model_spec_id,
        device_profile,
        deadline_ms,
        cancellation,
        exit,
    } = start;
    let session_id = uuid::Uuid::new_v4().to_string();
    let work_directory = app
        .path()
        .app_cache_dir()
        .map_err(|error| error.to_string())?
        .join("inference")
        .join(&session_id);
    let mut model_paths = RuntimeModelPaths {
        embedding: None,
        speech_transcription: None,
        speech_alignment: None,
        speech_diarization: None,
        speech_synthesis: None,
        speech_separation: None,
        image_segmentation: None,
        image_processing: None,
    };
    let value = Some(model_path.to_string_lossy().into_owned());
    match capability {
        RuntimeCapability::SpeechTranscription => model_paths.speech_transcription = value,
        RuntimeCapability::SpeechAlignment => model_paths.speech_alignment = value,
        RuntimeCapability::SpeechDiarization => model_paths.speech_diarization = value,
        RuntimeCapability::SpeechSynthesis => model_paths.speech_synthesis = value,
        RuntimeCapability::SpeechSeparation => model_paths.speech_separation = value,
        _ => return Err("runtime_mismatch: capability is not a speech method".into()),
    }
    let params = RuntimeInitializeParams {
        runtime_manifest_path: runtime.manifest_path.to_string_lossy().into_owned(),
        protocol_version: PROTOCOL_VERSION,
        runtime_digest: runtime.digest.clone(),
        session_id,
        capability,
        capability_spec_id: model_spec_id,
        device_profile,
        work_directory: work_directory.to_string_lossy().into_owned(),
        model_paths,
    };
    *slot = Some(
        Worker::start(
            runtime,
            &work_directory,
            model_path,
            params,
            deadline_ms,
            cancellation,
            exit,
        )
        .await?,
    );
    Ok(work_directory)
}
