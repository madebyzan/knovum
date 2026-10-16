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

#[test]
fn split_requests_use_explicit_identity_and_reject_unknown_fields() {
    let mut current = request(serde_json::json!({}));
    let decoded: super::SplitImageLayersRequest =
        serde_json::from_value(current.clone()).expect("current request");
    super::validate(&decoded).expect("valid split");
    assert_eq!(decoded.id, "run");
    assert_eq!(decoded.output_node_id, "run-output");
    let object = current.as_object_mut().expect("object");
    object.insert("outputNodeIds".into(), serde_json::json!(["legacy"]));
    assert!(serde_json::from_value::<super::SplitImageLayersRequest>(current.clone()).is_err());
    let object = current.as_object_mut().expect("object");
    object.remove("outputNodeIds");
    object.remove("id");
    assert!(serde_json::from_value::<super::SplitImageLayersRequest>(current).is_err());
}

#[test]
fn a_split_without_the_node_the_run_reserved_is_rejected() {
    let decoded: super::SplitImageLayersRequest =
        serde_json::from_value(request(serde_json::json!({"outputNodeId": " "}))).expect("request");
    assert!(super::validate(&decoded).is_err());
}

#[test]
fn a_split_rejects_a_source_edit_that_is_not_an_edit() {
    let decoded: super::SplitImageLayersRequest =
        serde_json::from_value(request(serde_json::json!({"sourceEdit": "crop"})))
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
fn the_first_layer_publishes_into_the_node_the_run_reserved() {
    let decoded: super::SplitImageLayersRequest =
        serde_json::from_value(request(serde_json::json!({}))).expect("request");
    assert_eq!(super::layer_node_id(&decoded, 0), "run-output");
    assert_eq!(super::layer_node_id(&decoded, 1), "run-layer-1");
    assert_eq!(super::derivation_id("run", 3), "run:layer-3");
    let ids = (0..super::MAX_LAYER_COUNT)
        .map(|index| super::layer_node_id(&decoded, index))
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
