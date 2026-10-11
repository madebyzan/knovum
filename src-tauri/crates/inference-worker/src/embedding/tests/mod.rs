use super::*;

#[test]
fn obsolete_embedding_identity_or_unadmitted_threads_fail_before_model_loading() {
    let mut params: RuntimeInitializeParams = serde_json::from_value(serde_json::json!({
        "runtimeManifestPath": "/inference/manifest.json", "runtimeDigest": "digest",
        "protocolVersion": 1, "sessionId": "session", "capability": "embedding",
        "capabilitySpecId": EMBEDDING_SPEC_ID,
        "deviceProfile": {"device": "cpu", "executionProfileId": "cpu-f32-threads4-native-v2", "cpuThreads": 4},
        "workDirectory": "/work", "modelPaths": {"embedding": "/not-loaded"}
    })).unwrap();
    assert!(Embedding::new(&params).is_ok());
    params.capability_spec_id = "obsolete-python-embedding".into();
    assert!(Embedding::new(&params).is_err());
    params.capability_spec_id = EMBEDDING_SPEC_ID.into();
    params.device_profile.cpu_threads = 0;
    assert!(Embedding::new(&params).is_err());
    params.device_profile.cpu_threads = 4;
    params.device_profile.execution_profile_id = "cpu-unknown".into();
    assert!(Embedding::new(&params).is_err());
}

#[test]
fn role_prompt_and_normalization_are_explicit() {
    assert_eq!(prepare("原生搜索", EmbeddingRole::Document), "原生搜索");
    assert_eq!(
        prepare("原生搜索", EmbeddingRole::Query),
        format!("{QUERY_PROMPT}原生搜索")
    );
    let mut vector = vec![0.0; 1024];
    vector[0] = 3.0;
    vector[1] = 4.0;
    normalize(&mut vector).unwrap();
    assert!((vector[0] - 0.6).abs() < 1e-7);
    assert!((vector[1] - 0.8).abs() < 1e-7);
    assert!(normalize(&mut vec![0.0; 1024]).is_err());
    vector[12] = f32::NAN;
    assert!(normalize(&mut vector).is_err());
    assert!(normalize(&mut [1.0; 1000]).is_err());
}
