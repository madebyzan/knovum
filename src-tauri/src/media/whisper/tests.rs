use super::*;
use std::{fs, os::unix::fs::PermissionsExt};

#[test]
fn local_whisper_executes_without_model_connections_or_credentials() {
    let root = std::env::temp_dir().join(format!(
        "knovum-local-transcription-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir(&root).unwrap();
    let executable = root.join("whisper-fixture");
    fs::write(&executable, r#"#!/bin/sh
while test "$#" -gt 0; do
  if test "$1" = -of; then shift; result=$1; fi
  shift
done
printf '%s' '{"result":{"language":"en"},"transcription":[{"offsets":{"from":0,"to":1000},"text":"Local evidence","tokens":[{"text":"Local evidence","offsets":{"from":0,"to":1000}}]}]}' > "$result.json"
"#).unwrap();
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).unwrap();
    let model = root.join("model.bin");
    let vad = root.join("vad.bin");
    let wav = root.join("source.wav");
    let output = root.join("transcript");
    for path in [&model, &vad, &wav] {
        fs::write(path, b"local fixture").unwrap();
    }
    let request = WhisperRequest {
        job_id: "local-only",
        executable: &executable,
        model: &model,
        vad_model: &vad,
        wav: &wav,
        output_base: &output,
        threads: 1,
        duration_ms: 1000,
    };
    let transcript = transcribe(request, &CancellationToken::new(), |_| Ok(())).unwrap();
    assert_eq!(transcript.language, "en");
    assert_eq!(transcript.segments[0].text, "Local evidence");
    assert_eq!(transcript.segments[0].end, 1.0);
    let missing = root.join("unavailable-whisper");
    let request = WhisperRequest {
        job_id: "missing-tool",
        executable: &missing,
        model: &model,
        vad_model: &vad,
        wav: &wav,
        output_base: &output,
        threads: 1,
        duration_ms: 1000,
    };
    assert!(transcribe(request, &CancellationToken::new(), |_| Ok(()))
        .err()
        .unwrap()
        .starts_with("failed to launch whisper.cpp CLI:"));
    fs::remove_dir_all(root).unwrap();
}
