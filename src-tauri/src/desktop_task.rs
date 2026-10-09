#[derive(Clone, Debug, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct DesktopTask {
    pub id: String,
    pub state: DesktopTaskState,
    pub progress: Option<u8>,
    pub started_at: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub(crate) enum DesktopTaskState {
    Queued,
    Running,
    Attention,
    Succeeded,
    Failed,
    Cancelled,
}

impl DesktopTaskState {
    pub(crate) fn active(self) -> bool {
        matches!(self, Self::Queued | Self::Running | Self::Attention)
    }
}
