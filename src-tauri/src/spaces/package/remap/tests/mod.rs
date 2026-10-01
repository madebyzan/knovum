use super::*;

fn manifest() -> Value {
    let mut manifest = json!({"format":"knovum-space", "version":super::super::PACKAGE_VERSION, "exportedAt":1,
        "space":{"id":"original", "kind":"space", "title":"Imported", "archivedAt":null, "lastOpenedAt":1, "createdAt":1, "updatedAt":1}});
    for collection in COLLECTIONS {
        manifest[*collection] = json!([]);
    }
    manifest
}

#[test]
fn rejects_unknown_manifest_fields_and_invalid_timestamps() {
    let mut value = manifest();
    value.as_object_mut().unwrap().remove("exportedAt");
    value["unexpected"] = json!(1);
    assert!(prepare(&value.to_string(), &[], "new").is_err());
    let mut value = manifest();
    value["exportedAt"] = json!(-1);
    assert!(prepare(&value.to_string(), &[], "new").is_err());
}

#[test]
fn rejects_missing_or_foreign_row_ownership_before_restoring() {
    for owner in [None, Some("foreign")] {
        let mut value = manifest();
        let mut relation = json!({"id":"relation"});
        if let Some(owner) = owner {
            relation["spaceId"] = json!(owner);
        }
        value["relations"] = json!([relation]);
        let error = prepare(&value.to_string(), &[], "new").unwrap_err();
        assert_eq!(error, "Package row belongs to another Space");
    }
}

#[test]
fn rejects_duplicate_identities() {
    let mut value = manifest();
    value["nodes"] = json!([{"id":"duplicate"}, {"id":"duplicate"}]);
    assert!(prepare(&value.to_string(), &[], "new")
        .unwrap_err()
        .contains("Duplicate package nodes"));
}

#[test]
fn restores_only_current_contract_with_new_space_identity() {
    let restored = prepare(&manifest().to_string(), &[], "new").unwrap();
    assert_eq!(restored.space.id, "new");
    assert_eq!(restored.space.title, "Imported");
    let mut value = manifest();
    value["version"] = json!(super::super::PACKAGE_VERSION - 1);
    assert!(prepare(&value.to_string(), &[], "new").is_err());
}

#[test]
fn restores_custom_type_metadata_without_system_or_source_ownership() {
    let mut value = manifest();
    value["objectTypes"] = json!([{"id":"custom","spaceId":"original","system":false,"name":"Example","icon":"box","description":"","fields":[],"createdAt":1,"updatedAt":1}]);
    let restored = prepare(&value.to_string(), &[], "new").unwrap();
    assert_ne!(restored.object_types[0].id, "custom");
    assert_eq!(restored.object_types[0].name, "Example");
}
