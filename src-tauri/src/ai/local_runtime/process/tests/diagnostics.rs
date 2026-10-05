use serde_json::{json, Value};

use super::decode;

fn diagnostic() -> Value {
    json!({
        "event": "requestFailed", "code": "invalid_output", "method": "speech.transcribe",
        "requestId": "34b3e359-01fd-41c5-80e9-999d316d9aaa", "exceptionType": "builtins.ValueError",
        "frames": [{ "file": "transcription.py", "function": "transcribe", "line": 42 }]
    })
}

#[test]
fn accepts_structural_failure_evidence() {
    let value = diagnostic();
    let parsed = decode(&serde_json::to_vec(&value).unwrap()).unwrap();
    assert_eq!(serde_json::to_value(parsed).unwrap(), value);
}

#[test]
fn rejects_unstructured_or_sensitive_fields() {
    for field in ["message", "source", "locals"] {
        let mut value = diagnostic();
        value[field] = json!("private content");
        assert!(decode(&serde_json::to_vec(&value).unwrap()).is_none());
    }
    let mut value = diagnostic();
    value["frames"][0]["file"] = json!("/private/user/transcription.py");
    assert!(decode(&serde_json::to_vec(&value).unwrap()).is_none());
}

#[test]
fn rejects_invalid_identity_and_unbounded_frames() {
    let mut value = diagnostic();
    value["requestId"] = json!("secret input");
    assert!(decode(&serde_json::to_vec(&value).unwrap()).is_none());
    value["requestId"] = Value::Null;
    value["frames"] = json!(vec![value["frames"][0].clone(); 9]);
    assert!(decode(&serde_json::to_vec(&value).unwrap()).is_none());
}
