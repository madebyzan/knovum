use super::SubmitConversationTaskRequest;
use serde_json::json;

#[test]
fn agent_requests_accept_both_targets_and_reject_removed_modes() {
    for target in [
        json!({"type":"model","connectionId":"connection","modelId":"model","temperature":0.2}),
        json!({"type":"agent","runtimeKind":"codex"}),
    ] {
        let mut request = json!({"spaceId":"space","prompt":"Explain these notes","target":target,
            "extensions":{"skillIds":[],"connections":[]}});
        assert!(serde_json::from_value::<SubmitConversationTaskRequest>(request.clone()).is_ok());
        for mode in ["ask", "plan", "execute"] {
            request["mode"] = json!(mode);
            let error = serde_json::from_value::<SubmitConversationTaskRequest>(request.clone())
                .unwrap_err();
            assert!(error.to_string().contains("unknown field `mode`"));
        }
    }
}
