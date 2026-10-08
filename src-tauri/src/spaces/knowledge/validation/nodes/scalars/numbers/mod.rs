use serde_json::{Map, Value};

const JAVASCRIPT_MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;

pub(in crate::spaces::knowledge::validation::nodes) fn is_javascript_safe_number(
    value: &Value,
) -> bool {
    value.as_f64().is_some_and(|number| {
        number.is_finite() && (number.fract() != 0.0 || number.abs() <= JAVASCRIPT_MAX_SAFE_INTEGER)
    })
}

pub(in crate::spaces::knowledge::validation::nodes) fn optional_integer_range(
    object: &Map<String, Value>,
    key: &str,
    min: i64,
    max: i64,
) -> Result<(), String> {
    let Some(value) = object.get(key) else {
        return Ok(());
    };
    let number = value
        .as_i64()
        .ok_or_else(|| format!("Space node data.{key} must be an integer"))?;
    if !(min..=max).contains(&number) {
        return Err(format!(
            "Space node data.{key} is outside the allowed range"
        ));
    }
    Ok(())
}

pub(in crate::spaces::knowledge::validation::nodes) fn optional_safe_nonnegative_integer(
    object: &Map<String, Value>,
    key: &str,
) -> Result<(), String> {
    let Some(value) = object.get(key) else {
        return Ok(());
    };
    safe_nonnegative_integer(value, key).map(|_| ())
}

pub(in crate::spaces::knowledge::validation::nodes) fn required_safe_nonnegative_integer(
    object: &Map<String, Value>,
    key: &str,
) -> Result<i64, String> {
    let value = object
        .get(key)
        .ok_or_else(|| format!("Space node data.{key} must be a non-negative safe integer"))?;
    safe_nonnegative_integer(value, key)
}

fn safe_nonnegative_integer(value: &Value, key: &str) -> Result<i64, String> {
    const JAVASCRIPT_MAX_SAFE_INTEGER: i64 = 9_007_199_254_740_991;
    let number = value
        .as_i64()
        .ok_or_else(|| format!("Space node data.{key} must be a non-negative safe integer"))?;
    if !(0..=JAVASCRIPT_MAX_SAFE_INTEGER).contains(&number) {
        return Err(format!(
            "Space node data.{key} must be a non-negative safe integer"
        ));
    }
    Ok(number)
}

#[cfg(test)]
mod tests;
