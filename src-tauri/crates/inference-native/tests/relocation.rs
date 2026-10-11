use super::dependency_owner;
#[test]
fn overlapping_ggml_names_resolve_to_their_engine_owner() {
    let audio = "bin/libggml.dylib".to_owned();
    let whisper = "compiled-libraries/whisper/libggml.dylib".to_owned();
    let libraries = [&audio, &whisper];
    assert_eq!(
        dependency_owner(
            "compiled-libraries/audiocpp.dylib",
            "@rpath/libggml.dylib",
            &libraries
        )
        .unwrap(),
        audio
    );
    assert_eq!(
        dependency_owner(
            "compiled-libraries/whisper/libwhisper.dylib",
            "@rpath/libggml.dylib",
            &libraries
        )
        .unwrap(),
        whisper
    );
    assert!(dependency_owner(
        "compiled-libraries/audiocpp.dylib",
        "libmissing.dylib",
        &libraries
    )
    .is_err());
}
