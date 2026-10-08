use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::RuntimeProfile;
use tauri::{AppHandle, Manager, Runtime};

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Descriptor {
    #[cfg_attr(
        debug_assertions,
        expect(
            dead_code,
            reason = "Only released runtime archives use publication fields"
        )
    )]
    pub(super) schema_version: u8,
    pub(super) target: String,
    pub(super) version: String,
    pub(super) dependency_requirements_sha256: String,
    pub(super) profile: RuntimeProfile,
    pub(super) dependency_lock_sha256: String,
    #[cfg_attr(
        debug_assertions,
        expect(
            dead_code,
            reason = "Only released runtime archives use publication fields"
        )
    )]
    pub(super) archive_url: String,
    pub(super) size: u64,
    pub(super) sha256: String,
    pub(super) manifest_sha256: String,
}

#[cfg(not(debug_assertions))]
fn validate_archive_url(value: &str) -> Result<(), String> {
    let url = reqwest::Url::parse(value).map_err(|error| error.to_string())?;
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return Err("python-runtime-invalid: archive URL is invalid".into());
    }
    Ok(())
}

#[cfg(not(debug_assertions))]
pub(super) fn embedded(profile: RuntimeProfile) -> Result<Option<Descriptor>, String> {
    let descriptor: Option<Descriptor> = serde_json::from_slice(profile.descriptor_bytes())
        .map_err(|error| format!("python-runtime-invalid: descriptor: {error}"))?;
    if let Some(value) = &descriptor {
        validate_archive_url(&value.archive_url)?;
        if value.schema_version != 1
            || value.profile != profile
            || value.target != crate::local_resources::tools::platform_directory()
            || value.version != "3.11.17"
            || value.size == 0
            || value.size > 4 * 1024 * 1024 * 1024
            || [
                &value.sha256,
                &value.manifest_sha256,
                &value.dependency_requirements_sha256,
                &value.dependency_lock_sha256,
            ]
            .iter()
            .any(|hash| !valid_hash(hash, 64))
        {
            return Err("python-runtime-invalid: invalid platform descriptor".into());
        }
    }
    Ok(descriptor)
}

#[cfg(not(debug_assertions))]
pub(super) fn required(profile: RuntimeProfile) -> Result<Descriptor, String> {
    embedded(profile)?
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
        .join(value.profile.name())
        .join(&value.target)
        .join(&value.sha256))
}

#[cfg(not(debug_assertions))]
fn valid_hash(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[derive(Serialize, ts_rs::TS)]
#[serde(tag = "state", rename_all = "snake_case")]
pub(crate) enum RuntimeStatus {
    Ready,
    NotPublished,
    NotInstalled {
        #[serde(rename = "downloadSize")]
        download_size: u64,
    },
    Invalid {
        message: String,
    },
}
