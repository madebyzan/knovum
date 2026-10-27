use super::*;

#[test]
fn a_desktop_agent_reads_source_files_itself_and_is_never_told_to_stay_off_them() {
    // Codex obeyed "never look on the local filesystem" over its attachment's path and summarised
    // a slide deck kept as pictures from its metadata alone.
    let policy = desktop_agent(false);
    assert!(policy.contains("open the source file with your own tools"));
    assert!(policy.contains("get_node_asset"));
    assert!(!policy.contains("only access"));
    assert!(!policy.contains("never look for Space data"));
}

#[test]
fn skill_discovery_is_asked_for_only_when_skills_are_selected() {
    assert!(!desktop_agent(false).contains("list_selected_skills"));
    assert!(desktop_agent(true).contains("list_selected_skills"));
}
