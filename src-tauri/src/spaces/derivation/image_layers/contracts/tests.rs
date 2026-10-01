#[test]
fn generation_request_uses_explicit_identity_and_rejects_legacy_fields() {
    let mut request = serde_json::json!({
        "generationId": "generation", "spaceId": "space", "sourceNodeId": "source",
        "sourceRevision": "revision", "sourceEdit": null,
        "outputNodeIds": ["layer-one", "layer-two"], "prompt": null
    });
    let decoded: super::GenerateImageLayersRequest =
        serde_json::from_value(request.clone()).expect("current request");
    super::validate(&decoded).expect("valid generation");
    assert_eq!(decoded.generation_id, "generation");
    request
        .as_object_mut()
        .expect("object")
        .insert("id".into(), "legacy".into());
    assert!(serde_json::from_value::<super::GenerateImageLayersRequest>(request.clone()).is_err());
    request
        .as_object_mut()
        .expect("object")
        .remove("generationId");
    assert!(serde_json::from_value::<super::GenerateImageLayersRequest>(request).is_err());
}
