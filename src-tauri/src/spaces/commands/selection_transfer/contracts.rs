use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferSelectionRequest {
    pub source_space_id: String,
    pub target_space_id: String,
    pub ids: Vec<String>,
}
