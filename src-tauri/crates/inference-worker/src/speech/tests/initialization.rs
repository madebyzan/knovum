use knovum_inference_contracts::runtime::RuntimeInitializeParams;

fn params(spec: &str, execution: &str, model: &str) -> RuntimeInitializeParams {
    serde_json::from_value(serde_json::json!({
        "runtimeManifestPath": "/inference/manifest.json", "runtimeDigest": "digest",
        "protocolVersion": 1, "sessionId": "session", "capability": "speech_transcription",
        "capabilitySpecId": spec,
        "deviceProfile": {"device": "cpu", "executionProfileId": execution, "cpuThreads": 4},
        "workDirectory": "/work", "modelPaths": {model: "/not-loaded"}
    }))
    .unwrap()
}

#[test]
fn transcription_rejects_an_execution_identity_that_does_not_match_its_model() {
    for variant in ["tiny-q8_0", "small-q8_0", "large-v3-f16"] {
        let (model, precision) = variant.rsplit_once('-').unwrap();
        let spec = format!("whisper-{model}-cpu-{precision}-native-v2");
        let mut input = params(&spec, &spec, "speechTranscription");
        assert!(super::super::transcription::Transcriber::new(&input).is_ok());
        input.device_profile.execution_profile_id = "obsolete-whisper".into();
        assert!(super::super::transcription::Transcriber::new(&input).is_err());
        input.device_profile.execution_profile_id = spec;
        input.model_paths.speech_transcription = None;
        assert!(super::super::transcription::Transcriber::new(&input).is_err());
    }
}

#[test]
fn vox_rejects_unadmitted_profiles_before_loading_a_inference_library() {
    for quantization in ["orig", "q8_0"] {
        let spec = format!("voxcpm2-2b-gguf-{quantization}-v1");
        let profile = format!("voxcpm2-cpu-gguf-{quantization}-v1");
        let mut input = params(&spec, &profile, "speechSynthesis");
        input.capability = knovum_inference_contracts::runtime::RuntimeCapability::SpeechSynthesis;
        assert!(super::super::voxcpm::VoxCpm::new(&input).is_ok());
        input.device_profile.execution_profile_id = "obsolete-vox".into();
        assert!(super::super::voxcpm::VoxCpm::new(&input).is_err());
        input.device_profile.execution_profile_id = profile;
        input.device_profile.cpu_threads = 0;
        assert!(super::super::voxcpm::VoxCpm::new(&input).is_err());
        input.device_profile.cpu_threads = 4;
        input.model_paths.speech_synthesis = None;
        assert!(super::super::voxcpm::VoxCpm::new(&input).is_err());
    }
}

#[test]
fn moss_rejects_a_wrong_profile_or_zero_threads_before_graph_loading() {
    let mut input = params(
        "moss-tts-nano-100m-onnx-native-v2",
        "moss-nano-cpu-f32-threads4-native-v2",
        "speechSynthesis",
    );
    assert!(super::super::moss::Moss::new(&input).is_ok());
    input.device_profile.execution_profile_id = "obsolete-moss".into();
    assert!(super::super::moss::Moss::new(&input).is_err());
    input.device_profile.execution_profile_id = "moss-nano-cpu-f32-threads4-native-v2".into();
    input.device_profile.cpu_threads = 0;
    assert!(super::super::moss::Moss::new(&input).is_err());
    input.device_profile.cpu_threads = 4;
    input.model_paths.speech_synthesis = None;
    assert!(super::super::moss::Moss::new(&input).is_err());
}

#[test]
fn sherpa_rejects_a_profile_that_differs_from_its_frozen_precision() {
    for spec in ["spleeter-2stems-fp16-cpu-v1", "spleeter-2stems-fp32-cpu-v1"] {
        let mut input = params(spec, spec, "speechSeparation");
        assert!(super::super::separation::Separator::new(&input).is_ok());
        input.device_profile.execution_profile_id = "other-precision".into();
        assert!(super::super::separation::Separator::new(&input).is_err());
    }
    for spec in [
        "sherpa-diarization-cpu-v1",
        "sherpa-diarization-fp32-cpu-v1",
    ] {
        let mut input = params(spec, spec, "speechDiarization");
        assert!(super::super::diarization::Diarizer::new(&input).is_ok());
        input.device_profile.execution_profile_id = "other-precision".into();
        assert!(super::super::diarization::Diarizer::new(&input).is_err());
    }
}
