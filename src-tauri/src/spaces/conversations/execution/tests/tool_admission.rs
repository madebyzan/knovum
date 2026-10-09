use super::super::gateway::require_tool;
use crate::spaces::conversations::tools;

#[test]
fn main_agent_can_plan_and_write_while_research_cannot_expand_authorization() {
    let catalog = tools::project().unwrap();
    for name in ["write_todos", "write_file", "edit_file", "task"] {
        assert!(require_tool(&catalog, name, "").is_ok());
        assert_eq!(
            require_tool(&catalog, name, "task:research|tools").unwrap_err(),
            "builtin-agent-tool-not-allowed"
        );
    }
    for name in ["read_file", "glob", "grep", "compact_conversation"] {
        assert!(require_tool(&catalog, name, "task:research|tools").is_ok());
    }
    for (name, binding) in &catalog.binding_by_wire_name {
        assert!(require_tool(&catalog, name, "").is_ok());
        assert_eq!(
            require_tool(&catalog, name, "task:research|tools").is_ok(),
            binding.read_only
        );
    }
    for name in ["unselected_tool", "execute", "delete"] {
        assert!(require_tool(&catalog, name, "").is_err());
    }
}
