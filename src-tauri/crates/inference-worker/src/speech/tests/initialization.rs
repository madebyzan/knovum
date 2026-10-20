use knovum_inference_contracts::{
    execution_profile::{moss_nano, whisper, WHISPER_VARIANTS},
    image_compute::ImageModelVariant,
    runtime::{RuntimeCapability, RuntimeInitializeParams, RuntimeModel},
};

fn params(spec: &str, execution: &str) -> RuntimeInitializeParams {
    serde_json::from_value(serde_json::json!({
        "runtimeManifestPath": "/inference/manifest.json", "runtimeDigest": "digest",
        "protocolVersion": 1, "sessionId": "session", "capability": "speech_transcription",
        "capabilitySpecId": spec,
        "deviceProfile": {"device": "cpu", "executionProfileId": execution, "cpuThreads": 4},
        "workDirectory": "/work", "model": {"kind": "snapshot", "path": "/not-loaded"},
        "budgetMs": 60000
    }))
    .unwrap()
}

/// The model a speech worker is handed is the snapshot its capability names, never an image
/// variant, so every speech engine rejects one the same way.
fn image_variant() -> RuntimeModel {
    RuntimeModel::ImageVariant {
        path: "/not-loaded".into(),
        variant: ImageModelVariant::Migan,
    }
}

#[test]
fn transcription_rejects_an_execution_identity_that_does_not_match_its_model() {
    for (model, precision) in WHISPER_VARIANTS {
        let spec = whisper(model, "cpu", precision);
        let mut input = params(&spec, &spec);
        assert_eq!(
            super::super::transcription::admit(&input).unwrap(),
            "/not-loaded"
        );
        input.device_profile.execution_profile_id = "obsolete-whisper".into();
        assert!(super::super::transcription::admit(&input).is_err());
        input.device_profile.execution_profile_id = spec;
        input.model = image_variant();
        assert!(super::super::transcription::admit(&input).is_err());
    }
}

#[test]
fn vox_rejects_unadmitted_profiles_before_loading_a_inference_library() {
    for quantization in ["orig", "q8_0"] {
        let spec = format!("voxcpm2-2b-gguf-{quantization}-v1");
        let profile = format!("voxcpm2-cpu-gguf-{quantization}-v1");
        let mut input = params(&spec, &profile);
        input.capability = RuntimeCapability::SpeechSynthesis;
        assert_eq!(
            super::super::voxcpm::admit(&input).unwrap(),
            (
                if quantization == "orig" {
                    "voxcpm2-orig.gguf"
                } else {
                    "voxcpm2-q8_0.gguf"
                },
                "/not-loaded"
            )
        );
        input.device_profile.execution_profile_id = "obsolete-vox".into();
        assert!(super::super::voxcpm::admit(&input).is_err());
        input.device_profile.execution_profile_id = profile;
        input.device_profile.cpu_threads = 0;
        assert!(super::super::voxcpm::admit(&input).is_err());
        input.device_profile.cpu_threads = 4;
        input.model = image_variant();
        assert!(super::super::voxcpm::admit(&input).is_err());
    }
}

#[test]
fn moss_rejects_a_wrong_profile_or_zero_threads_before_graph_loading() {
    let mut input = params("moss-tts-nano-100m-onnx-bundled-v2", &moss_nano(4));
    assert_eq!(super::super::moss::admit(&input).unwrap(), "/not-loaded");
    input.device_profile.execution_profile_id = "obsolete-moss".into();
    assert!(super::super::moss::admit(&input).is_err());
    input.device_profile.execution_profile_id = moss_nano(4);
    input.device_profile.cpu_threads = 0;
    assert!(super::super::moss::admit(&input).is_err());
    input.device_profile.cpu_threads = 4;
    input.model = image_variant();
    assert!(super::super::moss::admit(&input).is_err());
}

#[test]
fn sherpa_rejects_a_profile_that_differs_from_its_frozen_precision() {
    for spec in ["spleeter-2stems-fp16-cpu-v1", "spleeter-2stems-fp32-cpu-v1"] {
        let mut input = params(spec, spec);
        assert_eq!(
            super::super::separation::admit(&input).unwrap(),
            "/not-loaded"
        );
        input.device_profile.execution_profile_id = "other-precision".into();
        assert!(super::super::separation::admit(&input).is_err());
    }
    for spec in [
        "sherpa-diarization-cpu-v1",
        "sherpa-diarization-fp32-cpu-v1",
    ] {
        let mut input = params(spec, spec);
        assert_eq!(
            super::super::diarization::admit(&input).unwrap(),
            "/not-loaded"
        );
        input.device_profile.execution_profile_id = "other-precision".into();
        assert!(super::super::diarization::admit(&input).is_err());
    }
}
