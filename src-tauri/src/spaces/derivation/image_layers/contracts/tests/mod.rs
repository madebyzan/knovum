#[test]
fn plan_requests_use_explicit_identity_and_reject_unknown_fields() {
    let mut request = serde_json::json!({
        "planId": "plan", "spaceId": "space", "sourceNodeId": "source",
        "sourceRevision": "revision", "sourceEdit": null
    });
    let decoded: super::PlanImageLayersRequest =
        serde_json::from_value(request.clone()).expect("current request");
    super::validate_plan(&decoded).expect("valid plan");
    assert_eq!(decoded.plan_id, "plan");
    let object = request.as_object_mut().expect("object");
    object.insert("prompt".into(), "legacy".into());
    assert!(serde_json::from_value::<super::PlanImageLayersRequest>(request.clone()).is_err());
    request.as_object_mut().expect("object").remove("prompt");
    request.as_object_mut().expect("object").remove("planId");
    assert!(serde_json::from_value::<super::PlanImageLayersRequest>(request).is_err());
}

#[test]
fn generation_requests_bind_a_plan_to_unique_output_nodes() {
    let request = |outputs: serde_json::Value| {
        serde_json::json!({
            "generationId": "generation", "planId": "plan", "spaceId": "space",
            "sourceNodeId": "source", "sourceRevision": "revision", "sourceEdit": null,
            "outputNodeIds": outputs
        })
    };
    let decoded: super::GenerateImageLayersRequest =
        serde_json::from_value(request(serde_json::json!(["one", "two"]))).expect("request");
    super::validate(&decoded).expect("valid generation");
    for outputs in [
        serde_json::json!(["one"]),
        serde_json::json!(["one", "one"]),
        serde_json::json!(["one", "source"]),
    ] {
        let decoded: super::GenerateImageLayersRequest =
            serde_json::from_value(request(outputs)).expect("request");
        assert!(super::validate(&decoded).is_err());
    }
}

#[test]
fn the_output_ceiling_admits_a_discovered_sheet_of_subjects() {
    let outputs = (0..super::MAX_LAYER_COUNT)
        .map(|index| format!("layer-{index}"))
        .collect::<Vec<_>>();
    let decoded: super::GenerateImageLayersRequest = serde_json::from_value(serde_json::json!({
        "generationId": "generation", "planId": "plan", "spaceId": "space",
        "sourceNodeId": "source", "sourceRevision": "revision", "sourceEdit": null,
        "outputNodeIds": outputs
    }))
    .expect("request");
    super::validate(&decoded).expect("the ceiling is inclusive");
    assert!(
        super::MAX_LAYER_COUNT >= 10,
        "a nine subject sheet plus its backdrop must fit"
    );
}
