use serde_json::{Map, Value};

mod numbers;
mod text;

pub(super) use numbers::{
    is_javascript_safe_number, optional_integer_range, optional_safe_nonnegative_integer,
    required_safe_nonnegative_integer,
};
pub(super) use text::{
    optional_string, optional_string_allow_empty, optional_string_array, required_string,
    required_string_allow_empty,
};

pub(in crate::spaces::knowledge::validation::nodes) fn optional_bool(
    object: &Map<String, Value>,
    key: &str,
) -> Result<(), String> {
    if object.get(key).is_some_and(|value| !value.is_boolean()) {
        return Err(format!("Space node data.{key} must be boolean"));
    }
    Ok(())
}
