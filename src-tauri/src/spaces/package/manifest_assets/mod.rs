use std::collections::{HashMap, HashSet};

use serde_json::Value;
use tokio_util::sync::CancellationToken;

use super::validation;
use crate::spaces::assets::{visit_asset_references, AssetReference, PACKAGE_ASSET_PREFIX};

pub(super) struct ManifestAssets {
    pub used_tokens: HashSet<String>,
    pub source_revisions: HashMap<String, String>,
}

pub(super) fn parse_cancellable(
    manifest: &str,
    cancellation: &CancellationToken,
) -> Result<ManifestAssets, crate::app_error::AppError> {
    super::control::check(cancellation)?;
    let value: Value = serde_json::from_str(manifest).map_err(|error| error.to_string())?;
    let mut used_tokens = HashSet::new();
    let mut source_revisions = HashMap::new();
    for node in value
        .get("nodes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        super::control::check(cancellation)?;
        let Some(data) = node.get("data") else {
            continue;
        };
        // The traversal lends each reference for the duration of the callback, so they are owned here
        // before the fallible token work runs.
        let mut references: Vec<(AssetReference, String)> = Vec::new();
        visit_asset_references(data, &mut |kind, reference| {
            references.push((kind, reference.to_owned()));
        });
        for (kind, reference) in &references {
            let Some(token) = package_token(reference)? else {
                continue;
            };
            used_tokens.insert(token.to_owned());
            if *kind == AssetReference::Source {
                record_source_revision(&mut source_revisions, token, data)?;
            }
        }
    }
    for layout in value
        .get("layouts")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        super::control::check(cancellation)?;
        let Some(reference) = layout
            .get("appearance")
            .and_then(|appearance| appearance.get("background"))
            .and_then(Value::as_str)
        else {
            continue;
        };
        if let Some(token) = package_token(reference)? {
            used_tokens.insert(token.to_owned());
        }
    }
    collect_speech_assets(
        &value,
        &mut used_tokens,
        &mut source_revisions,
        cancellation,
    )?;
    Ok(ManifestAssets {
        used_tokens,
        source_revisions,
    })
}

fn collect_speech_assets(
    value: &Value,
    used_tokens: &mut HashSet<String>,
    source_revisions: &mut HashMap<String, String>,
    cancellation: &CancellationToken,
) -> Result<(), crate::app_error::AppError> {
    for separation in value
        .get("speechSourceSeparations")
        .and_then(Value::as_array)
        .ok_or("Space package speechSourceSeparations must be an array")?
    {
        super::control::check(cancellation)?;
        for (path, digest) in [
            ("sourceAssetPath", "sourceRevision"),
            ("backgroundAssetPath", "backgroundSha256"),
            ("vocalsAssetPath", "vocalsSha256"),
        ] {
            let reference = separation
                .get(path)
                .and_then(Value::as_str)
                .ok_or("Package source separation asset is unavailable")?;
            let token = package_token(reference)?
                .ok_or("Package source separation must use a managed asset token")?;
            let raw = separation
                .get(digest)
                .and_then(Value::as_str)
                .ok_or("Package source separation digest is unavailable")?;
            let revision = if digest != "sourceRevision" {
                format!("sha256:{raw}")
            } else {
                raw.to_owned()
            };
            if !valid_revision(&revision) {
                return Err("Package source separation digest is invalid".into());
            }
            record_digest(used_tokens, source_revisions, token, &revision)?;
        }
    }
    for revision in value
        .get("voiceRevisions")
        .and_then(Value::as_array)
        .ok_or("Space package voiceRevisions must be an array")?
    {
        super::control::check(cancellation)?;
        let Some(reference) = revision.get("referenceAssetId").and_then(Value::as_str) else {
            continue;
        };
        let token = package_token(reference)?
            .ok_or("Package voice reference must use a managed asset token")?;
        let digest = revision
            .get("referenceSourceRevision")
            .and_then(Value::as_str)
            .filter(|value| valid_revision(value))
            .ok_or("Package voice reference requires a SHA-256 source revision")?;
        record_digest(used_tokens, source_revisions, token, digest)?;
    }
    for render in value
        .get("speechRenders")
        .and_then(Value::as_array)
        .ok_or("Space package speechRenders must be an array")?
    {
        super::control::check(cancellation)?;
        let Some(reference) = render.get("assetPath").and_then(Value::as_str) else {
            continue;
        };
        let token = package_token(reference)?
            .ok_or("Package speech render must use a managed asset token")?;
        let digest = render
            .get("outputSha256")
            .and_then(Value::as_str)
            .filter(|value| value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
            .map(|value| format!("sha256:{value}"))
            .ok_or("Package speech render requires a SHA-256 output digest")?;
        record_digest(used_tokens, source_revisions, token, &digest)?;
    }
    for revision in value
        .get("speechProjectRevisions")
        .and_then(Value::as_array)
        .ok_or("Space package speechProjectRevisions must be an array")?
    {
        super::control::check(cancellation)?;
        if let Some(config) = revision
            .get("outputConfig")
            .filter(|config| config.get("video").is_some())
        {
            let config = crate::spaces::speech::video_config::parse(config)?;
            for source in std::iter::once(&config.video).chain(config.background.iter()) {
                let token = package_token(&source.asset_path)?
                    .ok_or("Package video dubbing sources require managed asset tokens")?;
                record_digest(
                    used_tokens,
                    source_revisions,
                    token,
                    &source.source_revision,
                )?;
            }
        }
        if let Some(cover) = revision
            .get("outputConfig")
            .and_then(|config| config.get("metadata"))
            .and_then(|metadata| metadata.get("cover"))
            .filter(|value| !value.is_null())
        {
            let reference = cover
                .get("assetPath")
                .and_then(Value::as_str)
                .ok_or("Package audiobook cover asset is unavailable")?;
            let token = package_token(reference)?
                .ok_or("Package audiobook cover requires a managed asset token")?;
            let revision = cover
                .get("sourceRevision")
                .and_then(Value::as_str)
                .filter(|value| valid_revision(value))
                .ok_or("Package audiobook cover requires an exact content revision")?;
            record_digest(used_tokens, source_revisions, token, revision)?;
        }
        let Some(reference) = revision.get("outputAssetPath").and_then(Value::as_str) else {
            continue;
        };
        let token = package_token(reference)?
            .ok_or("Package speech project output must use a managed asset token")?;
        let digest = revision
            .get("outputSha256")
            .and_then(Value::as_str)
            .filter(|value| value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
            .map(|value| format!("sha256:{value}"))
            .ok_or("Package speech project output requires a SHA-256 digest")?;
        record_digest(used_tokens, source_revisions, token, &digest)?;
    }
    Ok(())
}

fn record_digest(
    used_tokens: &mut HashSet<String>,
    revisions: &mut HashMap<String, String>,
    token: &str,
    digest: &str,
) -> Result<(), String> {
    used_tokens.insert(token.to_owned());
    if revisions
        .insert(token.to_owned(), digest.to_owned())
        .is_some_and(|current| current != digest)
    {
        return Err(
            "Space package reuses one asset token with conflicting source revisions".into(),
        );
    }
    Ok(())
}

fn valid_revision(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// Only the node's own managed asset carries the digest its `sourceRevision` records; an embedded
/// reference such as a favicon or an inline image describes something else entirely.
fn record_source_revision(
    revisions: &mut HashMap<String, String>,
    token: &str,
    payload: &Value,
) -> Result<(), String> {
    let revision = payload
        .get("sourceRevision")
        .and_then(Value::as_str)
        .filter(|value| value.starts_with("sha256:") && value.len() == 71)
        .ok_or("Managed package assets require a SHA-256 sourceRevision")?;
    if let Some(existing) = revisions.insert(token.to_owned(), revision.to_owned()) {
        if existing != revision {
            return Err(
                "Space package reuses one asset token with conflicting source revisions".into(),
            );
        }
    }
    Ok(())
}

fn package_token(reference: &str) -> Result<Option<&str>, String> {
    let Some(token) = reference.strip_prefix(PACKAGE_ASSET_PREFIX) else {
        return Ok(None);
    };
    validation::validate_token(token)?;
    Ok(Some(token))
}

#[cfg(test)]
mod tests;
