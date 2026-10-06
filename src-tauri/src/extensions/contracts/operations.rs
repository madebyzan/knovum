use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum PluginOperationStage {
    Reading,
    Validating,
    Committing,
}
