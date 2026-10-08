use super::*;
#[test]
fn desktop_mcp_rejects_inference_before_it_can_create_a_child_run() {
    for action in actions::definitions() {
        let lookup = require_platform_action(action.id);
        if action.routing_purpose.is_some() {
            assert!(lookup
                .err()
                .unwrap()
                .starts_with("agent-inference-reentry:"));
        } else {
            assert!(lookup.is_ok());
        }
    }
}

#[test]
fn external_agent_grant_ids_are_exactly_the_exposed_action_set() {
    let ids = external_agent_action_ids();
    let expected = actions::definitions()
        .iter()
        .filter(|action| action.routing_purpose.is_none())
        .map(|action| action.id.to_owned())
        .collect::<Vec<_>>();
    assert_eq!(ids, expected);
    assert!(actions::definitions()
        .iter()
        .filter(|action| action.routing_purpose.is_some())
        .all(|action| !ids.iter().any(|id| id == action.id)));
}
