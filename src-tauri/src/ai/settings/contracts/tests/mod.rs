use super::*;
use crate::ai::{contracts::RoutingPurpose, settings::validation};
use serde_json::json;

fn configured() -> serde_json::Value {
    json!({
        "schemaVersion": 1,
        "value": {
            "connections": [{
                "id": "user-model", "name": "My model", "provider": "openai",
                "baseUrl": "https://api.openai.com/v1",
                "models": [{"id": "text", "name": "Text", "model": "gpt-4.1", "capabilities": ["text-generation"]}]
            }],
            "desktopAgents": [{"kind": "codex", "enabled": true}],
            "defaults": {
                "writing": {"type": "model", "connectionId": "user-model", "modelId": "text"},
                "visualUnderstanding": {"type": "agent", "runtimeKind": "codex"}
            }
        }
    })
}

#[test]
fn absent_global_preference_preserves_accounts_and_explicit_overrides() {
    let envelope: Envelope<AiSettings> = serde_json::from_value(configured()).unwrap();
    assert_eq!(envelope.schema_version, 1);
    let settings = envelope.value;
    validation::validate(&settings).unwrap();
    assert_eq!(settings.connections[0].id, "user-model");
    assert_eq!(settings.desktop_agents[0].kind, ExternalAgentKind::Codex);
    assert!(settings.desktop_agents[0].enabled);
    assert!(settings.defaults.default_target.is_none());
    assert_eq!(settings.defaults.target(RoutingPurpose::Knowledge), None);
    assert_eq!(
        settings.defaults.target(RoutingPurpose::Writing),
        Some(TaskTarget::Model {
            connection_id: "user-model".into(),
            model_id: "text".into(),
        })
    );
    assert_eq!(
        settings
            .defaults
            .target(RoutingPurpose::VisualUnderstanding),
        Some(TaskTarget::Agent {
            runtime_kind: ExternalAgentKind::Codex,
        })
    );
}

#[test]
fn absent_overrides_inherit_only_an_explicit_global_preference() {
    let mut raw = configured();
    raw["value"]["defaults"] = json!({});
    let envelope: Envelope<AiSettings> = serde_json::from_value(raw.clone()).unwrap();
    validation::validate(&envelope.value).unwrap();
    assert_eq!(
        envelope.value.defaults.target(RoutingPurpose::Knowledge),
        None
    );
    raw["value"]["defaults"] = json!({"defaultTarget": {"type": "agent", "runtimeKind": "codex"}});
    let envelope: Envelope<AiSettings> = serde_json::from_value(raw).unwrap();
    validation::validate(&envelope.value).unwrap();
    assert_eq!(
        envelope.value.defaults.target(RoutingPurpose::Knowledge),
        Some(TaskTarget::Agent {
            runtime_kind: ExternalAgentKind::Codex,
        })
    );
}

#[test]
fn invalid_explicit_preferences_are_not_replaced_with_defaults() {
    for field in [
        "defaultTarget",
        "writing",
        "knowledge",
        "visualUnderstanding",
        "translation",
        "imageGeneration",
        "videoGeneration",
    ] {
        let mut raw = configured();
        raw["value"]["defaults"][field] = json!({"type": "agent", "runtimeKind": "unknown"});
        assert!(serde_json::from_value::<Envelope<AiSettings>>(raw).is_err());
    }
    let mut raw = configured();
    raw["value"]["defaults"]["defaultTarget"] =
        json!({"type": "agent", "runtimeKind": "qoder_cli"});
    let envelope: Envelope<AiSettings> = serde_json::from_value(raw).unwrap();
    assert!(validation::validate(&envelope.value).is_err());
}

#[test]
fn unknown_fields_and_missing_required_data_remain_invalid() {
    let mut unknown = configured();
    unknown["value"]["defaults"]["extra"] = json!(null);
    assert!(serde_json::from_value::<Envelope<AiSettings>>(unknown).is_err());
    for field in ["connections", "desktopAgents", "defaults"] {
        let mut raw = configured();
        raw["value"].as_object_mut().unwrap().remove(field);
        assert!(serde_json::from_value::<Envelope<AiSettings>>(raw).is_err());
    }
}
