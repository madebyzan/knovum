use serde_json::{json, Value};

use super::super::diagnostics::decode;

fn diagnostic() -> Value {
    json!({
        "event": "agent_runtime_request_failed",
        "method": "agentRuntime.execute",
        "exceptionType": "builtins.ValueError",
        "frames": [{ "file": "transport.py", "line": 42 }]
    })
}

#[test]
fn accepts_only_bounded_agent_runtime_diagnostics() {
    assert!(decode(&serde_json::to_vec(&diagnostic()).unwrap()).is_some());

    let mut too_many_frames = diagnostic();
    too_many_frames["frames"] = json!(vec![too_many_frames["frames"][0].clone(); 9]);
    assert!(decode(&serde_json::to_vec(&too_many_frames).unwrap()).is_none());
}

#[test]
fn rejects_unknown_methods_paths_and_sensitive_fields() {
    for (path, value) in [
        ("method", json!("agentRuntime.secretMethod")),
        ("exceptionType", json!("bad exception text")),
    ] {
        let mut candidate = diagnostic();
        candidate[path] = value;
        assert!(decode(&serde_json::to_vec(&candidate).unwrap()).is_none());
    }

    let mut path = diagnostic();
    path["frames"][0]["file"] = json!("/private/user/transport.py");
    assert!(decode(&serde_json::to_vec(&path).unwrap()).is_none());

    let mut secret = diagnostic();
    secret["message"] = json!("api-key-secret");
    assert!(decode(&serde_json::to_vec(&secret).unwrap()).is_none());
}

struct DiagnosticLog(std::sync::Mutex<Vec<String>>);
static DIAGNOSTIC_LOG: DiagnosticLog = DiagnosticLog(std::sync::Mutex::new(Vec::new()));

impl log::Log for DiagnosticLog {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata
            .target()
            .ends_with("agent_runtime::process::diagnostics")
    }
    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            self.0.lock().unwrap().push(record.args().to_string());
        }
    }
    fn flush(&self) {}
}

#[tokio::test]
async fn real_stderr_observer_logs_safe_frames_and_discards_raw_and_oversized_lines() {
    log::set_logger(&DIAGNOSTIC_LOG).unwrap();
    log::set_max_level(log::LevelFilter::Warn);
    let mut stream = b"provider api-key-secret\n".to_vec();
    stream.extend(vec![b'x'; 16 * 1024 + 1]);
    stream.push(b'\n');
    stream.extend(serde_json::to_vec(&diagnostic()).unwrap());
    stream.push(b'\n');
    let mut unsafe_diagnostic = diagnostic();
    unsafe_diagnostic["message"] = json!("api-key-secret");
    stream.extend(serde_json::to_vec(&unsafe_diagnostic).unwrap());
    super::super::diagnostics::observe(stream.as_slice()).await;
    let records = DIAGNOSTIC_LOG.0.lock().unwrap();
    assert_eq!(records.len(), 1);
    assert!(records[0].contains("transport.py"));
    assert!(records[0].contains("ValueError"));
    assert!(!records[0].contains("api-key-secret"));
}
