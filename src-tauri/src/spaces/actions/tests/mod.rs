use std::collections::HashSet;

use super::definitions;
mod image_transforms;

#[test]
fn registry_definitions_are_coherent() {
    let definitions = definitions();
    let ids = definitions
        .iter()
        .map(|definition| definition.id)
        .collect::<HashSet<_>>();

    assert_eq!(ids.len(), definitions.len());

    for definition in definitions {
        assert!(!definition.id.is_empty());
        assert!(definition.version > 0);
        let schema = super::parameter_schema(definition);
        let properties = schema
            .get("properties")
            .and_then(serde_json::Value::as_object);
        assert!(
            !definition.input_roles.is_empty()
                || properties.is_some_and(|properties| !properties.is_empty())
        );
        assert_eq!(
            definition.execution_capability.is_some(),
            definition.routing_purpose.is_some(),
        );

        let required = schema
            .get("required")
            .and_then(serde_json::Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let parameter_keys = required
            .iter()
            .filter_map(serde_json::Value::as_str)
            .collect::<HashSet<_>>();
        assert_eq!(parameter_keys.len(), required.len());
    }
}

#[test]
fn parameter_validation_rejects_unknown_and_blank_values() {
    let definition = super::require("create-note").expect("registered action");
    assert!(super::validate_parameters(
        definition,
        &serde_json::json!({
            "title": "Title", "content": "Body",
        })
    )
    .is_ok());
    assert!(super::validate_parameters(
        definition,
        &serde_json::json!({
            "title": "", "content": "Body",
        })
    )
    .is_err());
    assert!(super::validate_parameters(
        definition,
        &serde_json::json!({
            "title": "Title", "content": "Body", "unexpected": true,
        })
    )
    .is_err());
}

#[test]
fn parameter_contracts_are_json_schema_2020_12() {
    for definition in definitions() {
        let schema = super::parameter_schema(definition);
        assert_eq!(
            schema.get("$schema").and_then(serde_json::Value::as_str),
            Some("https://json-schema.org/draft/2020-12/schema"),
        );
    }
}
