use serde_json::{json, Map, Value};

/// One JSON Schema graph owns shared parameter types across the Action catalog.
pub(super) fn export() -> Result<Value, String> {
    let mut definitions = Map::new();
    let mut properties = Map::new();
    for action in crate::spaces::actions::definitions() {
        let mut schema = crate::spaces::actions::parameter_schema(action);
        let object = schema
            .as_object_mut()
            .ok_or("Action parameter schema must be an object")?;
        let title = object
            .get("title")
            .and_then(Value::as_str)
            .ok_or("Action parameter schema must name its type")?
            .to_owned();
        object.remove("$schema");
        if let Some(children) = object.remove("$defs") {
            for (name, child) in children
                .as_object()
                .ok_or("Action schema definitions must be an object")?
            {
                insert(&mut definitions, name, child.clone())?;
            }
        }
        insert(&mut definitions, &title, schema)?;
        properties.insert(action.id.into(), json!({"$ref":format!("#/$defs/{title}")}));
    }
    Ok(json!({
        "$schema":"https://json-schema.org/draft/2020-12/schema",
        "title":"SpaceActionParameters",
        "type":"object",
        "properties":properties,
        "$defs":definitions,
        "additionalProperties":false,
    }))
}

fn insert(definitions: &mut Map<String, Value>, name: &str, schema: Value) -> Result<(), String> {
    if let Some(existing) = definitions.get(name) {
        if *existing != schema {
            return Err(format!("Action schemas disagree on the type {name}"));
        }
    } else {
        definitions.insert(name.into(), schema);
    }
    Ok(())
}
