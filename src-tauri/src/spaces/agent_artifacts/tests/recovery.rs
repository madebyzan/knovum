use std::{
    collections::{HashMap, HashSet},
    fs,
};

use super::sweep;

#[test]
fn a_workspace_survives_only_while_its_space_and_conversation_exist() {
    let root =
        std::env::temp_dir().join(format!("knovum-agent-workspaces-{}", uuid::Uuid::new_v4()));
    let [live, deleted, conversation, retired]: [String; 4] =
        std::array::from_fn(|_| uuid::Uuid::new_v4().to_string());
    let kept = root.join(&live).join(&conversation);
    let unowned = root.join(&live).join(&retired);
    let orphaned = root.join(&deleted).join(&retired);
    for workspace in [&kept, &unowned, &orphaned] {
        fs::create_dir_all(workspace).expect("workspace");
        fs::write(workspace.join("output.md"), "draft").expect("Agent output");
    }
    let beside = root.join(&live).join(".DS_Store");
    fs::write(&beside, "").expect("file beside the sessions");
    let owners = HashMap::from([(live.clone(), HashSet::from([conversation.clone()]))]);

    sweep(&root, &owners).expect("sweep");

    assert!(
        kept.join("output.md").exists(),
        "a conversation keeps its workspace"
    );
    assert!(
        !unowned.exists(),
        "a session no conversation owns is dropped"
    );
    assert!(
        beside.exists(),
        "a file beside the sessions is not a workspace"
    );
    assert!(
        !root.join(&deleted).exists(),
        "a deleted Space loses its workspaces"
    );
    fs::remove_dir_all(&root).expect("cleanup");
}
