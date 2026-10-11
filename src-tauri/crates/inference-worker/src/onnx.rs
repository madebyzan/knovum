use crate::cancellation::Cancellation;
use ort::{
    session::{builder::GraphOptimizationLevel, Session},
    value::{DynValue, PrimitiveTensorElementType, Tensor},
};
use std::{collections::BTreeMap, path::Path};

pub(crate) type Values = BTreeMap<String, DynValue>;

pub(crate) fn session(path: &Path, threads: u8) -> Result<Session, String> {
    if !(1..=8).contains(&threads) {
        return Err("unsupported_device: ONNX thread count exceeds admission".into());
    }
    (|| -> ort::Result<Session> {
        Session::builder()?
            .with_execution_providers([ort::ep::CPU::default()
                .with_arena_allocator(true)
                .build()
                .error_on_failure()])?
            .with_intra_threads(usize::from(threads))?
            .with_inter_threads(1)?
            .with_parallel_execution(false)?
            .with_optimization_level(GraphOptimizationLevel::All)?
            .commit_from_file(path)
    })()
    .map_err(|error| format!("model_corrupt: {error}"))
}

pub(crate) fn tensor<T: PrimitiveTensorElementType + Clone + std::fmt::Debug + 'static>(
    shape: Vec<i64>,
    values: Vec<T>,
) -> Result<DynValue, String> {
    Tensor::from_array((shape, values))
        .map(|tensor| tensor.into_dyn())
        .map_err(|error| format!("invalid_input: {error}"))
}

pub(crate) fn run(
    session: &mut Session,
    feeds: Values,
    cancel: &Cancellation,
) -> Result<Values, String> {
    cancel.check()?;
    let options = cancel.run_options()?;
    let inputs = feeds.into_iter().collect::<Vec<_>>();
    let execution = session.run_with_options(inputs, &options);
    cancel.check()?;
    let outputs = execution.map_err(|error| format!("inference_failed: {error}"))?;
    Ok(outputs
        .into_iter()
        .map(|(name, value)| (name.to_owned(), value))
        .collect())
}

pub(crate) fn take(values: &mut Values, name: &str) -> Result<DynValue, String> {
    values
        .remove(name)
        .ok_or_else(|| format!("invalid_output: ONNX output {name} is absent"))
}
