use serde_json::{json, Value};

fn conformed(schema: Value) -> Value {
    let Value::Object(mut schema) = schema else {
        panic!("tool schemas are objects");
    };
    super::conform(&mut schema);
    Value::Object(schema)
}

#[test]
fn boolean_property_schemas_become_their_object_equivalents() {
    assert_eq!(
        conformed(json!({
            "type": "object",
            "properties": { "any": true, "none": false, "text": { "type": "string" } },
            "required": ["any"]
        })),
        json!({
            "type": "object",
            "properties": { "any": {}, "none": { "not": {} }, "text": { "type": "string" } },
            "required": ["any"]
        })
    );
}

#[test]
fn conformant_schemas_are_left_untouched() {
    let schema = json!({
        "type": "object",
        "properties": { "items": { "type": "array", "items": true } },
        "additionalProperties": true
    });
    assert_eq!(conformed(schema.clone()), schema);
    assert_eq!(
        conformed(json!({ "type": "object" })),
        json!({ "type": "object" })
    );
}
