use crate::spaces::derivation::image_operations::contracts::ImageTransformParameters;
use serde_json::Value;
use std::sync::OnceLock;

pub(in crate::spaces::actions::catalog) fn image_transform() -> &'static Value {
    static SCHEMA: OnceLock<Value> = OnceLock::new();
    SCHEMA.get_or_init(|| {
        serde_json::to_value(schemars::schema_for!(ImageTransformParameters))
            .expect("Image Action schema must serialize")
    })
}
