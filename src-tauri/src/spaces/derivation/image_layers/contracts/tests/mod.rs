#[test]
fn split_requests_use_explicit_identity_and_reject_unknown_fields() {
    let mut request = serde_json::json!({
        "id": "run", "spaceId": "space", "sourceNodeId": "source",
        "sourceRevision": "revision", "sourceEdit": null
    });
    let decoded: super::SplitImageLayersRequest =
        serde_json::from_value(request.clone()).expect("current request");
    super::validate(&decoded).expect("valid split");
    assert_eq!(decoded.id, "run");
    let object = request.as_object_mut().expect("object");
    object.insert("outputNodeIds".into(), serde_json::json!(["legacy"]));
    assert!(serde_json::from_value::<super::SplitImageLayersRequest>(request.clone()).is_err());
    request
        .as_object_mut()
        .expect("object")
        .remove("outputNodeIds");
    request.as_object_mut().expect("object").remove("id");
    assert!(serde_json::from_value::<super::SplitImageLayersRequest>(request).is_err());
}

#[test]
fn a_split_rejects_a_source_edit_that_is_not_an_edit() {
    let decoded: super::SplitImageLayersRequest = serde_json::from_value(serde_json::json!({
        "id": "run", "spaceId": "space", "sourceNodeId": "source",
        "sourceRevision": "revision", "sourceEdit": "crop"
    }))
    .expect("request");
    assert!(super::validate(&decoded).is_err());
}

#[test]
fn only_a_layer_stack_settles() {
    for count in [0, 1, super::MAX_LAYER_COUNT + 1] {
        assert!(super::validate_count(count).is_err());
    }
    for count in [super::MIN_LAYER_COUNT, super::MAX_LAYER_COUNT] {
        super::validate_count(count).expect("the bounds are inclusive");
    }
    assert!(
        super::MAX_LAYER_COUNT >= 10,
        "a nine subject sheet plus its backdrop must fit"
    );
}

#[test]
fn outputs_and_derivations_are_named_after_their_split() {
    assert_eq!(super::output_node_id("run", 0), "run-layer-0");
    assert_eq!(super::derivation_id("run", 3), "run:layer-3");
    let ids = (0..super::MAX_LAYER_COUNT)
        .map(|index| super::output_node_id("run", index))
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(
        ids.len(),
        super::MAX_LAYER_COUNT,
        "every output is distinct"
    );
    assert!(
        !ids.contains("run"),
        "no output collides with its source Run"
    );
}
