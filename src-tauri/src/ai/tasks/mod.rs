pub(crate) mod completion;
pub(crate) mod stream;
mod target;
pub(crate) mod text;

pub use target::{ExecutionSource, TaskExecutionTarget, TaskTarget, TaskTargetSelection};
