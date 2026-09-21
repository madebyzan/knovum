use super::validate_payload_shape;
use serde_json::json;

#[test]
fn directory_and_archive_node_kinds_are_not_space_contracts() {
    for kind in ["folder", "archive"] {
        let data = json!({"kind": kind, "title": "Project", "fileName": "Project", "assetPath": "/assets/project"});
        assert_eq!(
            validate_payload_shape(kind, &data).unwrap_err(),
            "Unsupported Space node kind"
        );
    }
}
