use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const MANIFEST_PATH: &str = "manifest.json";
pub const APPLICATION_ID: &str = "com.zand.knovum";
pub const ROOTS: &[&str] = &[
    "data/config/spaces.db",
    "data/config/extensions.db",
    "data/config/ai-usage.db",
    "data/store/settings.json",
    "data/assets/spaces",
    "data/assets/extensions",
];
pub const ARCHIVE_SCHEMA: u8 = 1;
pub const MAX_FILES: usize = 100_000;
pub const MAX_MANIFEST_BYTES: u64 = 32 * 1024 * 1024;
pub const MAX_FILE_BYTES: u64 = 16 * 1024 * 1024 * 1024;
pub const MAX_TOTAL_BYTES: u64 = 64 * 1024 * 1024 * 1024;

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArchiveFile {
    pub path: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Manifest {
    pub schema_version: u8,
    pub application_id: String,
    pub source_config_root: String,
    pub source_data_root: String,
    pub baselines: BTreeMap<String, String>,
    pub roots: BTreeMap<String, bool>,
    pub files: Vec<ArchiveFile>,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "operation", rename_all = "camelCase", deny_unknown_fields)]
pub enum PendingOperation {
    Backup { destination: String },
    Restore { source: String },
}

#[derive(Clone, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/settings/contracts/generated/")]
pub struct MaintenanceReceipt {
    pub operation: MaintenanceOperation,
    pub status: MaintenanceStatus,
    pub message: String,
    pub credentials_reconfiguration_required: bool,
    pub cleanup_warning: Option<String>,
}

#[derive(Clone, Copy, Deserialize, Serialize, schemars::JsonSchema, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/settings/contracts/generated/")]
pub enum MaintenanceOperation {
    Backup,
    Restore,
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, schemars::JsonSchema, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/settings/contracts/generated/")]
pub enum MaintenanceStatus {
    Success,
    Failed,
}

impl MaintenanceReceipt {
    pub fn success(
        operation: MaintenanceOperation,
        message: &str,
        cleanup_warning: Option<String>,
    ) -> Self {
        Self {
            operation,
            status: MaintenanceStatus::Success,
            message: message.into(),
            credentials_reconfiguration_required: matches!(
                operation,
                MaintenanceOperation::Restore
            ),
            cleanup_warning,
        }
    }

    pub fn failure(operation: MaintenanceOperation, message: String) -> Self {
        Self {
            operation,
            status: MaintenanceStatus::Failed,
            message,
            credentials_reconfiguration_required: false,
            cleanup_warning: None,
        }
    }
}
