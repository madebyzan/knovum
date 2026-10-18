use serde::Serialize;
use serde_json::Value;

/// What a derivation settled into the node it reserved. Lineage is not part of this: a derived
/// node's `derives-from` Relations are created when the node is reserved, so by the time it
/// settles they already exist on the canvas.
#[derive(Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase")]
pub struct CompletedDerivationOutput {
    pub node_id: String,
    pub data: Value,
}
