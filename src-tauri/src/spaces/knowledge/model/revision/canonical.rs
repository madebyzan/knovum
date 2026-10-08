use serde_json::Value;

fn canonical_number(value: &serde_json::Number) -> String {
    let number = value
        .as_f64()
        .expect("JSON numbers crossing the WebView boundary must convert to f64");
    let normalized = if number == 0.0 { 0.0 } else { number };
    format!("n{:016x}", normalized.to_bits())
}

pub(super) fn stable_json(value: &Value) -> String {
    match value {
        Value::Null => "null".into(),
        Value::Bool(value) => {
            if *value {
                "true".into()
            } else {
                "false".into()
            }
        }
        Value::Number(value) => canonical_number(value),
        Value::String(value) => {
            serde_json::to_string(value).expect("string serialization cannot fail")
        }
        Value::Array(values) => format!(
            "[{}]",
            values.iter().map(stable_json).collect::<Vec<_>>().join(",")
        ),
        Value::Object(object) => {
            let mut keys = object.keys().collect::<Vec<_>>();
            keys.sort();
            let fields = keys
                .into_iter()
                .map(|key| {
                    let encoded =
                        serde_json::to_string(key).expect("object key serialization cannot fail");
                    format!("{encoded}:{}", stable_json(&object[key]))
                })
                .collect::<Vec<_>>();
            format!("{{{}}}", fields.join(","))
        }
    }
}

use std::hash::Hasher;

pub(super) fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hasher = fnv::FnvHasher::default();
    hasher.write(bytes);
    hasher.finish()
}
