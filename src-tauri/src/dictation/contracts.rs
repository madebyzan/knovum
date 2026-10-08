use serde::Serialize;

#[derive(Debug)]
pub(crate) enum DictationFailure {
    Cancelled,
    Classified { code: &'static str, message: String },
}

impl DictationFailure {
    pub(crate) fn classified(code: &'static str, message: impl Into<String>) -> Self {
        Self::Classified {
            code,
            message: message.into(),
        }
    }

    pub(crate) fn into_parts(self) -> (&'static str, String) {
        match self {
            Self::Cancelled => ("cancelled", "Dictation was cancelled".into()),
            Self::Classified { code, message } => (code, message),
        }
    }
}

impl From<String> for DictationFailure {
    fn from(message: String) -> Self {
        Self::classified("dictation_failed", message)
    }
}

#[derive(Clone, Copy, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/dictation/contracts/generated/")]
pub(crate) enum DictationPhase {
    Idle,
    Recording,
    PreparingResources,
    Transcribing,
    Result,
    Failed,
}

#[derive(Clone, Copy, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/dictation/contracts/generated/")]
pub(crate) enum InsertionOutcome {
    Inserted,
    NotAttempted,
    FocusChanged,
    PermissionDenied,
    Failed,
}

#[derive(Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/dictation/contracts/generated/")]
pub(crate) struct DictationSnapshot {
    pub(crate) phase: DictationPhase,
    pub(crate) resources:
        Option<crate::local_resources::preparation::contracts::ResourcePreparationProgress>,
    pub(crate) session_id: Option<String>,
    pub(crate) started_at: Option<u64>,
    pub(crate) target_application: Option<String>,
    pub(crate) text: Option<String>,
    pub(crate) language: Option<String>,
    pub(crate) insertion: Option<InsertionOutcome>,
    pub(crate) error_code: Option<String>,
    pub(crate) error_message: Option<String>,
}

impl Default for DictationSnapshot {
    fn default() -> Self {
        Self {
            phase: DictationPhase::Idle,
            resources: None,
            session_id: None,
            started_at: None,
            target_application: None,
            text: None,
            language: None,
            insertion: None,
            error_code: None,
            error_message: None,
        }
    }
}

impl DictationSnapshot {
    pub(crate) fn failed(session_id: Option<String>, code: &str, message: String) -> Self {
        Self {
            phase: DictationPhase::Failed,
            session_id,
            error_code: Some(code.into()),
            error_message: Some(message),
            ..Self::default()
        }
    }
}
