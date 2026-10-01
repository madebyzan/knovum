
use super::*;
use serde_json::json;
fn configured() -> serde_json::Value {
    json!({
        "connections":[], "desktopAgents":[{"kind":"codex","enabled":true}],
        "defaults":{"defaultTarget":{"type":"agent","runtimeKind":"codex"},"writing":null,"knowledge":null,"visualUnderstanding":null,"translation":null,"imageGeneration":null,"videoGeneration":null}
    })
}
#[test]
fn native_model_contract_does_not_accept_an_agent_identity() {
    assert!(serde_json::from_value::<ModelTargetSelection>(
        json!({"type":"agent","runtimeKind":"codex"})
    )
    .is_err());
}
#[test]
fn explicit_override_precedes_the_default_execution_method() {
    let mut value = configured();
    value["defaults"]["knowledge"] = json!({"type":"agent","runtimeKind":"qoder_cli"});
    let settings: AiSettings = serde_json::from_value(value).unwrap();
    assert_eq!(
        settings
            .defaults
            .target(crate::ai::contracts::RoutingPurpose::Knowledge),
        Some(crate::ai::tasks::TaskTargetRef::Agent {
            runtime_kind: crate::ai::contracts::AgentKind::QoderCli
        })
    );
    assert_eq!(
        settings
            .defaults
            .target(crate::ai::contracts::RoutingPurpose::Writing),
        Some(crate::ai::tasks::TaskTargetRef::Agent {
            runtime_kind: crate::ai::contracts::AgentKind::Codex
        })
    );
}
#[test]
fn disabled_agent_is_unavailable_and_model_native_capabilities_remain_enforced() {
    let mut value = configured();
    value["desktopAgents"] = json!([]);
    let error = resolve_task_settings(
        serde_json::from_value(value).unwrap(),
        TaskTargetSelection::Agent {
            runtime_kind: crate::ai::contracts::AgentKind::Codex,
        },
        ExecutionCapability::VisionUnderstanding,
    )
    .unwrap_err();
    assert_eq!(
        error.code,
        super::super::error::AiErrorCode::AgentUnavailable
    );
    let mut value = configured();
    value["connections"] = json!([{"id":"byok","name":"User","provider":"openai","baseUrl":"https://api.openai.com/v1","models":[{"id":"text","name":"Text","model":"text","capabilities":["text-generation"]}]}]);
    let selection = TaskTargetSelection::Model {
        connection_id: "byok".into(),
        model_id: "text".into(),
        temperature: 0.2,
    };
    let target = resolve_task_settings(
        serde_json::from_value(value.clone()).unwrap(),
        selection.clone(),
        ExecutionCapability::TextGeneration,
    )
    .unwrap();
    assert!(matches!(target, TaskExecutionTarget::Model(_)));
    let error = resolve_task_settings(
        serde_json::from_value(value).unwrap(),
        selection,
        ExecutionCapability::VisionUnderstanding,
    )
    .unwrap_err();
    assert_eq!(
        error.code,
        super::super::error::AiErrorCode::CapabilityNotSupported
    );
}
