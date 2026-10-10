mod agent;
mod capabilities;
pub(crate) mod contracts;
mod execution;
mod local;
mod mask;
pub(crate) mod selection_preview;
pub(crate) use agent::discard_interrupted;
pub(super) mod output;
mod output_file;
mod persistence;
mod provider;
mod routing;
mod source;
mod validation;

pub use capabilities::capabilities;
pub use contracts::ImageOperationCapabilities;
pub(crate) use execution::derive_for_action;
