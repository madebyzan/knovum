
use super::*;
#[test]
fn desktop_mcp_rejects_inference_before_it_can_create_a_child_run() {
    for action in actions::definitions() {
        let result = require_platform_action(action.id);
        if action.routing_purpose.is_some() {
            assert!(result
                .err()
                .unwrap()
                .starts_with("agent-inference-reentry:"));
        } else {
            assert!(result.is_ok());
        }
    }
}
