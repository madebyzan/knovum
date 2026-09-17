use std::{
    io::Cursor,
    sync::{Arc, OnceLock},
};

use tract_onnx::prelude::*;

use super::pixels::{MODEL_PIXELS, MODEL_RESOLUTION};

static MODEL_PLAN: OnceLock<Result<Arc<TypedRunnableModel>, String>> = OnceLock::new();
static MODEL: &[u8] = include_bytes!("../models/u2netp.onnx");

pub fn run(input: Vec<f32>) -> Result<Vec<f32>, String> {
    let plan = MODEL_PLAN
        .get_or_init(create_model_plan)
        .as_ref()
        .map_err(Clone::clone)?;
    let tensor = Tensor::from_shape(&[1, 3, MODEL_RESOLUTION, MODEL_RESOLUTION], &input)
        .map_err(|error| error.to_string())?;
    let outputs = plan
        .run(tvec![tensor.into_tvalue()])
        .map_err(|error| error.to_string())?;
    let values = outputs[0]
        .to_plain_array_view::<f32>()
        .map_err(|error| error.to_string())?;
    if values.len() < MODEL_PIXELS {
        return Err("Background-removal model returned an invalid mask".into());
    }
    Ok(values.iter().take(MODEL_PIXELS).copied().collect())
}

fn create_model_plan() -> Result<Arc<TypedRunnableModel>, String> {
    let mut model = Cursor::new(MODEL);
    tract_onnx::onnx()
        .model_for_read(&mut model)
        .map_err(|error| error.to_string())?
        .with_input_fact(
            0,
            f32::fact([1, 3, MODEL_RESOLUTION, MODEL_RESOLUTION]).into(),
        )
        .map_err(|error| error.to_string())?
        .into_optimized()
        .map_err(|error| error.to_string())?
        .into_runnable()
        .map_err(|error| error.to_string())
}
