use super::*;
use serde_json::json;

#[test]
fn persists_explicit_region_in_current_strict_envelope() {
    let raw = serde_json::to_value(Envelope {
        schema_version: 1,
        value: ResourceDownloadRegion::China,
    })
    .unwrap();
    assert_eq!(decode(raw).unwrap(), ResourceDownloadRegion::China);
}

#[test]
fn invalid_preferences_fail_visibly_without_compatibility_reader() {
    for raw in [
        json!({"schemaVersion": 0, "value": "china"}),
        json!({"schemaVersion": 1, "value": "auto"}),
        json!({"schemaVersion": 1, "value": "global", "fallback": "china"}),
        json!("china"),
    ] {
        assert!(decode(raw).is_err());
    }
}
