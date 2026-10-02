use serde::Serialize;

#[derive(Clone, Debug, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMetadata {
    pub version: String,
    pub current_version: String,
    pub notes: String,
}

impl From<&tauri_plugin_updater::Update> for UpdateMetadata {
    fn from(update: &tauri_plugin_updater::Update) -> Self {
        Self {
            version: update.version.clone(),
            current_version: update.current_version.clone(),
            notes: update.body.clone().unwrap_or_default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum UpdateFailure {
    Check,
    Download,
    Signature,
    Install,
}

#[derive(Clone, Debug, Serialize, ts_rs::TS)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum UpdateState {
    Disabled,
    Idle,
    Checking,
    Current,
    Available {
        release: UpdateMetadata,
    },
    Downloading {
        release: UpdateMetadata,
        downloaded: u64,
        total: Option<u64>,
    },
    Verifying {
        release: UpdateMetadata,
    },
    Ready {
        release: UpdateMetadata,
    },
    Installing {
        release: UpdateMetadata,
    },
    Failed {
        reason: UpdateFailure,
    },
}

impl UpdateState {
    pub(super) fn can_check(&self) -> bool {
        matches!(self, Self::Idle | Self::Current | Self::Failed { .. })
    }

    pub(super) fn require_version(&self, version: &str, installing: bool) -> Result<(), String> {
        let release = match (self, installing) {
            (Self::Available { release }, false) | (Self::Ready { release }, true) => release,
            _ => return Err("The updater is not ready for this operation.".into()),
        };
        if release.version != version {
            return Err("The selected update has changed; review the current version.".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSnapshot {
    pub revision: u64,
    pub state: UpdateState,
}
