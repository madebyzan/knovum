use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Runtime};

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Descriptor {
    pub(super) schema_version: u8,
    pub(super) target: String,
    pub(super) version: String,
    pub(super) dependency_requirements_sha256: String,
    pub(super) voxcpm_commit: String,
    pub(super) url: String,
    pub(super) size: u64,
    pub(super) sha256: String,
    pub(super) manifest_sha256: String,
}

pub(super) fn embedded() -> Result<Option<Descriptor>, String> {
    let descriptor: Option<Descriptor> = serde_json::from_slice(include_bytes!(concat!(
        env!("OUT_DIR"),
        "/python-runtime-descriptor.json"
    )))
    .map_err(|error| format!("python-runtime-invalid: descriptor: {error}"))?;
    if let Some(value) = &descriptor {
        let url = reqwest::Url::parse(&value.url).map_err(|error| error.to_string())?;
        if value.schema_version != 1
            || value.target != crate::local_resources::tools::platform_directory()
            || value.version != "3.11.17"
            || url.scheme() != "https"
            || !url.username().is_empty()
            || url.password().is_some()
            || value.size == 0
            || value.size > 4 * 1024 * 1024 * 1024
            || [
                &value.sha256,
                &value.manifest_sha256,
                &value.dependency_requirements_sha256,
            ]
            .iter()
            .any(|hash| !valid_hash(hash, 64))
            || !valid_hash(&value.voxcpm_commit, 40)
        {
            return Err("python-runtime-invalid: invalid platform descriptor".into());
        }
    }
    Ok(descriptor)
}

pub(super) fn required() -> Result<Descriptor, String> {
    embedded()?
        .ok_or_else(|| "runtime_missing: Python runtime publication is not configured".into())
}

pub(super) fn destination<R: Runtime>(
    app: &AppHandle<R>,
    value: &Descriptor,
) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("local-resources/runtimes/python")
        .join(&value.target)
        .join(&value.sha256))
}

fn valid_hash(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RuntimeStatus {
    pub(crate) available: bool,
    pub(crate) downloadable: bool,
    pub(crate) download_size: Option<u64>,
}
