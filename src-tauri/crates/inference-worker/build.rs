use std::{env, path::PathBuf};
fn main() {
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo sets OUT_DIR"));
    for (file, name, functions) in [
        (
            "whisper/ggml-backend.h",
            "GgmlApi",
            r"ggml_backend_(load_all_from_path|dev_count|dev_get|dev_type)",
        ),
        ("voice.h", "VoiceApi", r"knovum_(voice_.*|audio_devices)"),
        (
            "whisper/whisper.h",
            "WhisperApi",
            r"whisper_(version|init_from_file_with_params_no_state|context_default_params|init_state|free|free_state|full_default_params|full_with_state|full_n_segments_from_state|full_get_segment_(t0|t1|text)_from_state|full_n_tokens_from_state|full_get_token_(text|data)_from_state|full_lang_id_from_state|token_eot|lang_id|lang_str|pcm_to_mel_with_state|lang_auto_detect_with_state|vad_.*)",
        ),
        (
            "sherpa-onnx.h",
            "SherpaApi",
            r"SherpaOnnx(GetVersionStr|CreateOfflineSourceSeparation|DestroyOfflineSourceSeparation|OfflineSourceSeparation.*|DestroySourceSeparationOutput|CreateOfflineSpeakerDiarization|DestroyOfflineSpeakerDiarization|OfflineSpeakerDiarization.*)",
        ),
        (
            "audiocpp.h",
            "AudioApi",
            r"audiocpp_(abi_version|last_error|registry_create|registry_free|model_load|model_free|session_create|session_free|session_run|request_create|request_free|request_set_(text|text_language|audio)|result_free|result_word_count|result_word)",
        ),
    ] {
        let header = format!("ffi-headers/{file}");
        println!("cargo:rerun-if-changed={header}");
        bindgen::Builder::default()
            .header(header)
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            .allowlist_function(functions)
            .dynamic_library_name(name)
            .dynamic_link_require_all(true)
            .generate_comments(false)
            .layout_tests(false)
            .generate()
            .expect("pinned C API bindings")
            .write_to_file(output.join(format!("{name}.rs")))
            .expect("write C API bindings");
    }
}
