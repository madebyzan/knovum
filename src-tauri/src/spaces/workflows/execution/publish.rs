use serde_json::{json, Value};
use sqlx::SqlitePool;
use tauri::AppHandle;

use super::{action::execute_invocation, shared::completed};
use crate::spaces::runs::{AttemptClaim, StepOutcome};

pub(super) async fn execute(
    app: &AppHandle,
    pool: &SqlitePool,
    claim: &AttemptClaim,
    mode: super::super::definition::PublicationMode,
    destination: Option<&str>,
) -> Result<StepOutcome, String> {
    if claim
        .policy_snapshot
        .get("testMode")
        .and_then(Value::as_bool)
        == Some(true)
    {
        return completed(json!({
            "isolated": true,
            "publicationMode": mode,
            "candidate": claim.resolved_input,
        }));
    }
    match mode {
        super::super::definition::PublicationMode::CreateResource => {
            let data = claim
                .resolved_input
                .get("data")
                .unwrap_or(&claim.resolved_input);
            let title = data
                .get("title")
                .and_then(Value::as_str)
                .ok_or("publish-input-invalid: title is required")?;
            let content = data
                .get("text")
                .or_else(|| data.get("plainText"))
                .and_then(Value::as_str)
                .ok_or("publish-input-invalid: editable text is required")?;
            execute_invocation(
                pool,
                claim,
                "create-note".into(),
                1,
                json!({ "title": title, "content": content }),
                Value::Null,
            )
            .await
        }
        super::super::definition::PublicationMode::UpdateExpectedRevision => {
            let operations = claim
                .resolved_input
                .get("operations")
                .cloned()
                .ok_or("publish-input-invalid: operations are required")?;
            execute_invocation(
                pool,
                claim,
                "commit-knowledge-changes".into(),
                1,
                json!({ "operations": operations }),
                Value::Null,
            )
            .await
        }
        super::super::definition::PublicationMode::Export => {
            export_asset(app, claim, destination).await
        }
    }
}

async fn export_asset(
    app: &AppHandle,
    claim: &AttemptClaim,
    destination: Option<&str>,
) -> Result<StepOutcome, String> {
    let source = claim
        .resolved_input
        .get("assetPath")
        .and_then(Value::as_str)
        .ok_or("publish-input-invalid: assetPath is required")?;
    let destination = destination
        .filter(|value| !value.trim().is_empty())
        .ok_or("publish-configuration-invalid: export destination is required")?;
    let source_path = std::path::Path::new(source)
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let managed_root = crate::spaces::assets::asset_directory(app)?
        .canonicalize()
        .map_err(|error| error.to_string())?;
    if !source_path.starts_with(&managed_root) {
        return Err("publish-input-invalid: export source is outside managed assets".into());
    }
    let metadata = tokio::fs::symlink_metadata(&source_path)
        .await
        .map_err(|error| error.to_string())?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("publish-input-invalid: export source must be a regular managed file".into());
    }
    let mut source_file = tokio::fs::File::open(&source_path)
        .await
        .map_err(|error| error.to_string())?;
    let mut destination_file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
        .await
        .map_err(|error| error.to_string())?;
    tokio::io::copy(&mut source_file, &mut destination_file)
        .await
        .map_err(|error| error.to_string())?;
    completed(json!({ "destination": destination, "bytes": metadata.len() }))
}
