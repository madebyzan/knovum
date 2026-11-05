use super::super::tests::EXAMPLE as SET;
use super::{validate, ImageSetRequest};

fn request(extra: serde_json::Value) -> serde_json::Value {
    let mut request = serde_json::json!({
        "id": "run", "spaceId": "space", "outputNodeId": "run-output",
        "sourceNodeId": "source", "sourceRevision": "revision", "sourceEdit": null
    });
    let object = request.as_object_mut().expect("object");
    for (key, value) in extra.as_object().expect("object") {
        object.insert(key.clone(), value.clone());
    }
    request
}

fn decoded(extra: serde_json::Value) -> ImageSetRequest {
    serde_json::from_value(request(extra)).expect("request")
}

#[test]
fn set_requests_use_explicit_identity_and_reject_unknown_fields() {
    let mut current = request(serde_json::json!({}));
    let parsed: ImageSetRequest = serde_json::from_value(current.clone()).expect("current request");
    validate(&parsed).expect("valid set");
    assert_eq!(parsed.id, "run");
    assert_eq!(parsed.output_node_id, "run-output");
    let object = current.as_object_mut().expect("object");
    object.insert("outputNodeIds".into(), serde_json::json!(["legacy"]));
    assert!(serde_json::from_value::<ImageSetRequest>(current.clone()).is_err());
    let object = current.as_object_mut().expect("object");
    object.remove("outputNodeIds");
    object.remove("id");
    assert!(serde_json::from_value::<ImageSetRequest>(current).is_err());
}

#[test]
fn a_set_without_the_node_the_run_reserved_is_rejected() {
    assert!(validate(&decoded(serde_json::json!({"outputNodeId": " "}))).is_err());
}

#[test]
fn a_set_rejects_a_source_edit_that_is_not_an_edit() {
    assert!(validate(&decoded(serde_json::json!({"sourceEdit": "crop"}))).is_err());
}

#[test]
fn only_a_count_the_set_describes_settles() {
    for count in [0, 1, 25] {
        assert_eq!(
            SET.validate_count(count).unwrap_err(),
            SET.unsupported,
            "{count} members"
        );
    }
    for count in [2, 24] {
        SET.validate_count(count).expect("the bounds are inclusive");
    }
}

#[test]
fn the_first_member_publishes_into_the_node_the_run_reserved() {
    let parsed = decoded(serde_json::json!({}));
    assert_eq!(SET.node_id(&parsed, 0), "run-output");
    assert_eq!(SET.node_id(&parsed, 1), "run-part-1");
    assert_eq!(SET.derivation_id("run", 3), "run:part-3");
    let ids = (0..*SET.members.end())
        .map(|index| SET.node_id(&parsed, index))
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(ids.len(), *SET.members.end(), "every output is distinct");
    assert!(
        !ids.contains("run"),
        "no output collides with its source Run"
    );
}
