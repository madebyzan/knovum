pub(crate) mod completion;
pub(crate) mod stream;
pub(crate) mod structured;
mod target;
pub(crate) mod text;

pub use target::{ExecutionSource, TaskExecutionTarget, TaskTarget, TaskTargetSelection};
