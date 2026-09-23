use super::super::digest::canonicalize;

#[test]
fn recursively_orders_object_keys() {
    let left = serde_json::json!({"z": {"b": 2, "a": 1}, "a": 0});
    let right = serde_json::json!({"a": 0, "z": {"a": 1, "b": 2}});
    assert_eq!(canonicalize(left), canonicalize(right));
}
