use super::super::WorkerSpecification;
use super::*;
use crate::ai::contracts::runtime::RuntimeModel;
use crate::local_resources::runtimes::inference::{InferenceRuntimePaths, SpawnedImage};

#[tokio::test]
async fn missing_inference_runtime_is_rejected_before_admission_or_workspace_creation() {
    let directory = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
    let runtime = InferenceRuntimePaths {
        manifest_path: directory.join("runtime-manifest.json"),
        executable: directory.join("knovum-inference-worker"),
        digest: "a".repeat(64),
        spawned_image: SpawnedImage::DeveloperBuild,
        library_bytes: 0,
    };
    let startup = Worker::start(
        &runtime,
        &directory,
        WorkerSpecification {
            capability: RuntimeCapability::Embedding,
            capability_spec_id: "qwen3-embedding-0.6b".into(),
            device_profile: RuntimeDeviceProfile {
                device: "cpu".into(),
                execution_profile_id: crate::ai::contracts::execution_profile::embedding("cpu", 4),
                cpu_threads: 4,
            },
            model: RuntimeModel::Snapshot {
                path: directory.join("model").to_string_lossy().into_owned(),
            },
        },
        &CancellationToken::new(),
        &CancellationToken::new(),
    )
    .await;
    assert!(matches!(startup, Err(error) if error.starts_with("runtime_missing:")));
    assert!(!directory.exists());
}
