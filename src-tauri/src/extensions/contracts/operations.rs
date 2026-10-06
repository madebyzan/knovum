use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum PluginOperationStage {
    Reading,
    Validating,
    Committing,
}

/// The value is already committed. Cleanup failures are separate from mutation failure.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExtensionChange<T> {
    pub value: T,
    pub cleanup_errors: Vec<String>,
}

impl<T> ExtensionChange<T> {
    pub(crate) fn committed(
        value: T,
        cleanup: impl IntoIterator<Item = Result<(), String>>,
    ) -> Self {
        Self {
            value,
            cleanup_errors: cleanup.into_iter().filter_map(Result::err).collect(),
        }
    }
}

#[cfg(test)]
#[path = "../tests/operations.rs"]
mod tests;
