use std::{collections::HashMap, path::Path};

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Manifest {
    pub schema_version: u8,
    pub model_id: String,
    pub license_review: LicenseReview,
    pub archive_url: String,
    pub archive_size: u64,
    pub archive_sha256: String,
    pub archive_root: String,
    pub files: Vec<ManifestFile>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct LicenseReview {
    pub status: String,
    pub reason: String,
    pub components: Vec<LicenseComponent>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct LicenseComponent {
    pub path: String,
    pub declared_license: String,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ManifestFile {
    pub path: String,
    pub size: u64,
    pub sha256: String,
}

pub(super) fn load() -> Result<Manifest, String> {
    let manifest: Manifest =
        serde_json::from_str(include_str!("../kokoro-int8-multi-lang-v1_0.json"))
            .map_err(|error| format!("Kokoro package manifest is invalid: {error}"))?;
    validate(&manifest)?;
    Ok(manifest)
}

fn validate(manifest: &Manifest) -> Result<(), String> {
    if manifest.schema_version != 1 || manifest.model_id != "kokoro-int8-multi-lang-v1_0" {
        return Err("Kokoro package manifest identity is invalid".to_string());
    }
    if manifest.license_review.status != "blocked"
        || manifest.license_review.reason.trim().is_empty()
        || manifest.license_review.components.is_empty()
        || manifest.license_review.components.iter().any(|component| {
            component.path.trim().is_empty() || component.declared_license.trim().is_empty()
        })
    {
        return Err("Kokoro package license review metadata is invalid".to_string());
    }
    reqwest::Url::parse(&manifest.archive_url)
        .map_err(|error| format!("Kokoro package archive URL is invalid: {error}"))?;
    if manifest.archive_size == 0
        || manifest.archive_sha256.len() != 64
        || manifest.archive_root != manifest.model_id
        || manifest.files.is_empty()
    {
        return Err("Kokoro package archive metadata is invalid".to_string());
    }
    let mut unique = HashMap::with_capacity(manifest.files.len());
    for file in &manifest.files {
        validate_relative_path(&file.path)?;
        if file.size == 0 || file.sha256.len() != 64 || unique.insert(&file.path, ()).is_some() {
            return Err(format!(
                "Kokoro package file metadata is invalid: {}",
                file.path
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_relative_path(path: &str) -> Result<(), String> {
    let path = Path::new(path);
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|part| !matches!(part, std::path::Component::Normal(_)))
    {
        return Err(format!(
            "Kokoro package contains an unsafe path: {}",
            path.display()
        ));
    }
    Ok(())
}
