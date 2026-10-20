use crate::extensions::contracts::{
    McpConnectionCatalog, McpResourceTemplateSelection, McpResourceTemplateValue,
};
use serde_json::json;
fn catalog() -> McpConnectionCatalog {
    McpConnectionCatalog {
        server: json!({}),
        tools: vec![],
        resources: vec![],
        resource_templates: vec![rmcp::model::ResourceTemplate::new(
            "https://example.com/items/{id}{?tags*}",
            "item",
        )],
        prompts: vec![serde_json::from_value(
            json!({"name":"draft","arguments":[{"name":"subject","required":true}]}),
        )
        .unwrap()],
        digest: "catalog".into(),
    }
}
#[test]
fn concrete_resource_selection_uses_rfc6570_and_rejects_undeclared_variables() {
    let mut selected = McpResourceTemplateSelection {
        uri_template: "https://example.com/items/{id}{?tags*}".into(),
        parameters: [
            ("id".into(), McpResourceTemplateValue::String("a/b".into())),
            (
                "tags".into(),
                McpResourceTemplateValue::List(vec!["one".into(), "two".into()]),
            ),
        ]
        .into_iter()
        .collect(),
    };
    assert_eq!(
        super::resource_selection::expand(&catalog(), &selected).unwrap(),
        "https://example.com/items/a%2Fb?tags=one&tags=two"
    );
    selected.parameters.insert(
        "undeclared".into(),
        McpResourceTemplateValue::String("value".into()),
    );
    assert!(super::resource_selection::expand(&catalog(), &selected).is_err());
}
#[test]
fn prompt_argument_authority_is_exact() {
    let schema = super::resource_selection::prompt_schema(&catalog(), &["draft".into()]);
    let validator = jsonschema::validator_for(&schema).unwrap();
    assert!(validator.is_valid(&json!({"name":"draft","arguments":{"subject":"topic"}})));
    for request in [
        json!({"name":"draft"}),
        json!({"name":"other","arguments":{"subject":"topic"}}),
        json!({"name":"draft","arguments":{"subject":"topic","extra":"value"}}),
    ] {
        assert!(!validator.is_valid(&request));
    }
}

#[test]
fn template_parameters_reject_values_outside_the_rfc6570_context_at_deserialization() {
    for value in [
        json!(1),
        json!(true),
        json!([null]),
        json!({"nested":{"value":"text"}}),
    ] {
        assert!(serde_json::from_value::<McpResourceTemplateValue>(value).is_err());
    }
    for value in [
        json!(null),
        json!("text"),
        json!(["text"]),
        json!({"key":"text"}),
    ] {
        assert!(serde_json::from_value::<McpResourceTemplateValue>(value).is_ok());
    }
}

#[test]
fn external_schema_references_do_not_open_files_or_network_endpoints() {
    for uri in [
        "file:///private/knovum/schema.json",
        "https://127.0.0.1:1/schema.json",
    ] {
        assert!(jsonschema::validator_for(&json!({"$ref":uri})).is_err());
    }
    let local = json!({"type":"object","$defs":{"value":{"type":"string"}},"properties":{"value":{"$ref":"#/$defs/value"}}});
    assert!(jsonschema::validator_for(&local)
        .unwrap()
        .is_valid(&json!({"value":"reviewed"})));
}

#[test]
fn runtime_resource_parameters_keep_template_identity_and_rfc6570_authority() {
    let catalog = catalog();
    let templates = vec![catalog.resource_templates[0].uri_template.clone()];
    let schema = super::resource_selection::resource_schema(&[], &templates).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    let arguments =
        json!({"uriTemplate":templates[0],"parameters":{"id":"a/b","tags":["one","two"]}});
    assert!(validator.is_valid(&arguments));
    assert_eq!(
        super::resource_selection::invocation_uri(&catalog, &[], &templates, arguments.clone())
            .unwrap(),
        "https://example.com/items/a%2Fb?tags=one&tags=two"
    );
    assert_eq!(
        super::resource_selection::invocation_uri(&catalog, &[], &[], arguments).unwrap_err(),
        "mcp-resource-template-not-authorized"
    );
    assert_eq!(
        super::resource_selection::invocation_uri(
            &catalog,
            &[],
            &templates,
            json!({"uri":"file:///private"})
        )
        .unwrap_err(),
        "mcp-resource-not-authorized"
    );
    for parameters in [
        json!({"unknown":"value"}),
        json!({"id":1}),
        json!({"tags":[true]}),
        json!({"id":{"key":2}}),
    ] {
        assert!(!validator.is_valid(&json!({"uriTemplate":templates[0], "parameters":parameters})));
    }
    let over_budget = json!({"uriTemplate":templates[0],"parameters":{"id":"a".repeat(65536)}});
    assert_eq!(
        super::resource_selection::invocation_uri(&catalog, &[], &templates, over_budget)
            .unwrap_err(),
        "mcp-resource-template-parameters-too-large"
    );
}
