use knovum_inference_contracts::{
    execution_profile::{moss_nano, separation, voice, whisper, WHISPER_VARIANTS},
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
    for (quantization, file) in [
        ("orig", "voxcpm2-orig.gguf"),
        ("bf16", "voxcpm2-bf16.gguf"),
        ("q8_0", "voxcpm2-q8_0.gguf"),
    ] {
        let spec = format!("voxcpm2-2b-gguf-{quantization}-v1");
        let profile = format!("voxcpm2-cpu-gguf-{quantization}-v1");
        let mut input = params(&spec, &profile);
        input.capability = RuntimeCapability::SpeechSynthesis;
        assert_eq!(
            super::super::voxcpm::admit(&input).unwrap(),
            (file, "/not-loaded")
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
fn separation_rejects_a_profile_that_differs_from_its_model_or_probed_device() {
    for (spec, file) in [
        ("htdemucs-q8_0-v1", "htdemucs-q8_0.gguf"),
        ("htdemucs-f16-v1", "htdemucs-f16.gguf"),
    ] {
        let mut input = params(spec, &separation(spec, "cpu").unwrap());
        input.capability = RuntimeCapability::SpeechSeparation;
        assert_eq!(
            super::super::separation::admit(&input).unwrap(),
            std::path::Path::new("/not-loaded").join(file)
        );
        input.device_profile.execution_profile_id = separation(spec, "mps").unwrap();
        assert!(super::super::separation::admit(&input).is_err());
        input.device_profile.execution_profile_id = separation(spec, "cpu").unwrap();
        input.model = image_variant();
        assert!(super::super::separation::admit(&input).is_err());
    }
}

#[test]
fn diarization_rejects_a_profile_that_differs_from_its_frozen_precision() {
    for spec in [
        "sherpa-diarization-cpu-v1",
        "sherpa-diarization-campplus-cpu-v1",
        "sherpa-diarization-eres2net-large-cpu-v1",
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

#[test]
fn cloning_rejects_a_profile_that_differs_from_its_model_or_probed_device() {
    for (spec, file) in [
        ("index-tts2-q8_0-v1", "index-tts2-q8_0.gguf"),
        (
            "qwen3-tts-1.7b-q8_0-v1",
            "qwen3-tts-12hz-1.7b-base-q8_0_v2.gguf",
        ),
        ("cosyvoice3-0.5b-q8_0-v1", "cosyvoice3-q8_0.gguf"),
    ] {
        let mut input = params(spec, &voice(spec, "cpu").unwrap());
        input.capability = RuntimeCapability::SpeechSynthesis;
        assert_eq!(
            super::super::cloning::admit(&input).unwrap().0,
            std::path::Path::new("/not-loaded").join(file)
        );
        input.device_profile.execution_profile_id = voice(spec, "vulkan").unwrap();
        assert!(super::super::cloning::admit(&input).is_err());
    }
}
