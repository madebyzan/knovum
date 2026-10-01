use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMetadata {
    pub version: String,
    pub current_version: String,
    pub notes: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum UpdatePhase {
    Idle,
    Checking,
    Available,
    Downloading,
    Verifying,
    Ready,
    Installing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum UpdateFailure {
    Network,
    Signature,
    Metadata,
    Installation,
    Internal,
}

#[derive(Clone, Debug, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSnapshot {
    pub revision: u32,
    pub configured: bool,
    pub phase: UpdatePhase,
    pub update: Option<UpdateMetadata>,
    #[ts(type = "number")]
    pub downloaded: u64,
    #[ts(type = "number | null")]
    pub total: Option<u64>,
    pub failure: Option<UpdateFailure>,
}

impl UpdateSnapshot {
    pub fn new(configured: bool) -> Self {
        Self {
            revision: 0,
            configured,
            phase: UpdatePhase::Idle,
            update: None,
            downloaded: 0,
            total: None,
            failure: None,
        }
    }
}
