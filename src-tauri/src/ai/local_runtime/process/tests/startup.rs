use super::*;
use crate::ai::contracts::runtime::{RuntimeInitializeParams, RuntimeModelPaths, PROTOCOL_VERSION};
use crate::local_resources::runtimes::python::{PythonRuntimePaths, RuntimeProfile};

#[tokio::test]
async fn wrong_runtime_package_is_rejected_before_admission_or_workspace_creation() {
    let directory = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
    let runtime = PythonRuntimePaths {
        profile: RuntimeProfile::Resources,
        manifest_path: directory.join("runtime-manifest.json"),
        executable: directory.join("python"),
        package_root: directory.join("packages"),
        worker_root: directory.join("worker"),
        digest: "a".repeat(64),
        lease: None,
    };
    let params = RuntimeInitializeParams {
        runtime_manifest_path: runtime.manifest_path.to_string_lossy().into_owned(),
        protocol_version: PROTOCOL_VERSION,
        runtime_digest: runtime.digest.clone(),
        session_id: uuid::Uuid::new_v4().to_string(),
        capability: RuntimeCapability::Embedding,
        capability_spec_id: "qwen3-embedding-0.6b".into(),
        device_profile: RuntimeDeviceProfile {
            device: "cpu".into(),
            execution_profile_id: "cpu-f32-threads4-v1".into(),
            cpu_threads: 4,
        },
        work_directory: directory.to_string_lossy().into_owned(),
        model_paths: RuntimeModelPaths {
            embedding: Some(directory.join("model").to_string_lossy().into_owned()),
            speech_transcription: None,
            speech_alignment: None,
            speech_diarization: None,
            speech_synthesis: None,
            speech_separation: None,
            image_segmentation: None,
        },
    };
    let result = Worker::start(
        &runtime,
        &directory,
        &directory.join("model"),
        params,
        deadline_after(5_000),
        &CancellationToken::new(),
        &CancellationToken::new(),
    )
    .await;
    assert!(
        matches!(result, Err(error) if error == "runtime_mismatch: worker package does not own the execution profile")
    );
    assert!(!directory.exists());
}
