use super::super::contracts::{
    AgentConnectionSelection, McpConnectionCatalog, McpResourceTemplateSelection,
    McpResourceTemplateValue,
};
use iri_string::{
    spec::UriSpec,
    template::{
        simple_context::{SimpleContext, Value as TemplateValue},
        UriTemplateStr,
    },
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn uris(
    catalog: &McpConnectionCatalog,
    selection: &AgentConnectionSelection,
) -> Result<Vec<String>, String> {
    let mut uris = BTreeSet::new();
    for uri in &selection.resource_uris {
        if !catalog
            .resources
            .iter()
            .any(|resource| &resource.uri == uri)
        {
            return Err("mcp-resource-not-in-catalog".into());
        }
        if !uris.insert(uri.clone()) {
            return Err("mcp-resource-selection-duplicated".into());
        }
    }
    for template in &selection.resource_templates {
        let uri = expand(catalog, template)?;
        if !uris.insert(uri) {
            return Err("mcp-resource-selection-duplicated".into());
        }
    }
    Ok(uris.into_iter().collect())
}

pub(crate) fn expand(
    catalog: &McpConnectionCatalog,
    selected: &McpResourceTemplateSelection,
) -> Result<String, String> {
    if !catalog
        .resource_templates
        .iter()
        .any(|template| template.uri_template == selected.uri_template)
    {
        return Err("mcp-resource-template-not-in-catalog".into());
    }
    let template =
        UriTemplateStr::new(&selected.uri_template).map_err(|_| "mcp-resource-template-invalid")?;
    let variables: BTreeSet<_> = template
        .variables()
        .map(|name| name.as_str().to_owned())
        .collect();
    if selected
        .parameters
        .keys()
        .any(|name| !variables.contains(name))
    {
        return Err("mcp-resource-template-parameter-unknown".into());
    }
    if serde_json::to_vec(&selected.parameters)
        .map_err(|error| error.to_string())?
        .len()
        > 65536
    {
        return Err("mcp-resource-template-parameters-too-large".into());
    }
    let mut context = SimpleContext::new();
    for (name, value) in &selected.parameters {
        context.insert(name.clone(), parameter(value));
    }
    let uri = template
        .expand::<UriSpec, _>(&context)
        .map_err(|_| "mcp-resource-template-expansion-failed")?
        .to_string();
    if uri.is_empty() || uri.len() > 8192 {
        return Err("mcp-resource-uri-size-invalid".into());
    }
    Ok(uri)
}

fn parameter(value: &McpResourceTemplateValue) -> TemplateValue {
    match value {
        McpResourceTemplateValue::Undefined(()) => TemplateValue::Undefined,
        McpResourceTemplateValue::String(value) => TemplateValue::String(value.clone()),
        McpResourceTemplateValue::List(values) => TemplateValue::List(values.clone()),
        McpResourceTemplateValue::Map(values) => TemplateValue::Assoc(
            values
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
        ),
    }
}

pub(crate) fn resource_schema(uris: &[String], templates: &[String]) -> Result<Value, String> {
    let mut variants = Vec::new();
    if !uris.is_empty() {
        variants.push(serde_json::json!({"type":"object","properties":{"uri":{"type":"string","enum":uris}},"required":["uri"],"additionalProperties":false}));
    }
    let value_schema = serde_json::json!({"oneOf":[
        {"type":"string"}, {"type":"array","items":{"type":"string"}},
        {"type":"object","additionalProperties":{"type":"string"}}, {"type":"null"}
    ]});
    for uri_template in templates {
        let template =
            UriTemplateStr::new(uri_template).map_err(|_| "mcp-resource-template-invalid")?;
        let properties: BTreeMap<_, _> = template
            .variables()
            .map(|name| (name.as_str(), value_schema.clone()))
            .collect();
        variants.push(serde_json::json!({"type":"object","properties":{
            "uriTemplate":{"const":uri_template},
            "parameters":{"type":"object","properties":properties,"additionalProperties":false}
        },"required":["uriTemplate","parameters"],"additionalProperties":false}));
    }
    Ok(serde_json::json!({"type":"object","oneOf":variants}))
}

pub(crate) fn invocation_uri(
    catalog: &McpConnectionCatalog,
    resource_uris: &[String],
    resource_templates: &[String],
    arguments: Value,
) -> Result<String, String> {
    if let Some(uri) = arguments.get("uri").and_then(Value::as_str) {
        if !resource_uris.iter().any(|selected| selected == uri) {
            return Err("mcp-resource-not-authorized".into());
        }
        return Ok(uri.into());
    }
    let selected: McpResourceTemplateSelection = serde_json::from_value(arguments)
        .map_err(|error| format!("mcp-resource-template-arguments-invalid: {error}"))?;
    if !resource_templates.contains(&selected.uri_template) {
        return Err("mcp-resource-template-not-authorized".into());
    }
    expand(catalog, &selected)
}

pub(crate) fn prompt_schema(catalog: &McpConnectionCatalog, names: &[String]) -> Value {
    let variants: Vec<_> = catalog.prompts.iter().filter(|prompt| names.contains(&prompt.name)).map(|prompt| {
        let arguments = prompt.arguments.as_deref().unwrap_or_default();
        let properties: BTreeMap<_,_> = arguments.iter().map(|argument| (&argument.name, serde_json::json!({"type":"string"}))).collect();
        let required: Vec<_> = arguments.iter().filter(|argument| argument.required == Some(true)).map(|argument| &argument.name).collect();
        serde_json::json!({"type":"object","properties":{"name":{"const":prompt.name},"arguments":{"type":"object","properties":properties,"required":required,"additionalProperties":false}},"required":if required.is_empty() { vec!["name"] } else { vec!["name","arguments"] },"additionalProperties":false})
    }).collect();
    serde_json::json!({"type":"object","oneOf":variants})
}
