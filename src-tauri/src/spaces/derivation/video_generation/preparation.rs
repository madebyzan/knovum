use std::path::Path;

use serde_json::Value;
use tauri::AppHandle;

use super::GenerateVideoRequest;
use crate::ai::models::ExecutionTarget;
use crate::spaces::derivation::{inputs, pending_output, source_validation};
use crate::spaces::{assets::asset_directory, db, knowledge::contracts};

pub struct PreparedVideo {
    pub path: String,
    pub reference_images: Vec<String>,
    pub provider_prompt: String,
}

pub async fn start(
    app: &AppHandle,
    request: &GenerateVideoRequest,
    target: &ExecutionTarget,
    authority: &super::super::action_execution::ExecutionAuthority,
) -> Result<PreparedVideo, String> {
    validate_request(request)?;
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    authority.require_active(&mut *tx).await?;
    let asset_directory = asset_directory(app)?;
    for input in &request.inputs {
        contracts::validate_same_space_node(
            &mut tx,
            &request.space_id,
            &input.node_id,
            &request.output_node_id,
        )
        .await?;
        source_validation::validate(
            &mut tx,
            &request.space_id,
            &input.node_id,
            &input.source_revision,
            input_basis(&input.role)?,
        )
        .await?;
    }
    let script = validate_primary(&mut tx, request).await?;
    let provider_prompt = super::prompt::assemble(&script, &request.prompt)?;
    let references = load_references(&mut tx, request, &asset_directory).await?;
    let path = output(&mut tx, request, &asset_directory).await?;
    persist_start(&mut tx, request, target).await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(PreparedVideo {
        path,
        reference_images: references,
        provider_prompt,
    })
}

fn validate_request(request: &GenerateVideoRequest) -> Result<(), String> {
    let primary_count = request
        .inputs
        .iter()
        .filter(|input| input.role == "primary")
        .count();
    if request.inputs.is_empty()
        || primary_count != 1
        || request
            .inputs
            .first()
            .is_none_or(|input| input.role != "primary")
        || request
            .inputs
            .iter()
            .skip(1)
            .any(|input| input.role != "source")
    {
        return Err("Video generation inputs are invalid".into());
    }
    Ok(())
}

async fn validate_primary(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    request: &GenerateVideoRequest,
) -> Result<String, String> {
    let primary = &request.inputs[0];
    let script: Option<String> = sqlx::query_scalar("SELECT json_extract(data,'$.text') FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND kind='markdown' AND deleted_at IS NULL AND json_extract(data,'$.knowledgeRevision')=$3")
        .bind(&primary.node_id).bind(&request.space_id).bind(&primary.source_revision).fetch_one(&mut **tx).await.map_err(db::sql_error)?;
    script
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "Video generation requires a current non-empty script".into())
}

async fn load_references(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    request: &GenerateVideoRequest,
    directory: &Path,
) -> Result<Vec<String>, String> {
    let mut values = Vec::with_capacity(request.inputs.len().saturating_sub(1));
    for input in request.inputs.iter().skip(1) {
        let encoded: String = sqlx::query_scalar("SELECT data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND kind='image' AND deleted_at IS NULL")
            .bind(&input.node_id).bind(&request.space_id).fetch_optional(&mut **tx).await.map_err(db::sql_error)?
            .ok_or("Video reference must be a current image")?;
        let data: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
        let path = Path::new(
            data["assetPath"]
                .as_str()
                .ok_or("Video reference path is unavailable")?,
        );
        if path.parent() != Some(directory) {
            return Err("Video reference is outside the Space asset directory".into());
        }
        let mime = mime_guess::from_path(path)
            .first_raw()
            .ok_or("Video reference image format is unsupported")?;
        if !matches!(
            mime,
            "image/jpeg" | "image/png" | "image/webp" | "image/bmp"
        ) {
            return Err("Video reference image format is unsupported".into());
        }
        values.push(path.to_string_lossy().into_owned());
    }
    Ok(values)
}

async fn output(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    request: &GenerateVideoRequest,
    directory: &Path,
) -> Result<String, String> {
    let encoded: String = sqlx::query_scalar(
        "SELECT data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND kind='video'",
    )
    .bind(&request.output_node_id)
    .bind(&request.space_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let data: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
    let path = pending_output::validate_unpublished_asset(&data)?.to_owned();
    let parent = Path::new(&path)
        .parent()
        .ok_or("Generated video output path is invalid")?
        .canonicalize()
        .map_err(|error| format!("Unable to resolve output directory: {error}"))?;
    let expected_dir = directory
        .canonicalize()
        .map_err(|error| format!("Unable to resolve asset directory: {error}"))?;
    if parent != expected_dir {
        return Err("Generated video output path is outside the Space asset directory".into());
    }
    Ok(path)
}

async fn persist_start(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    request: &GenerateVideoRequest,
    target: &ExecutionTarget,
) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    sqlx::query("INSERT INTO spaces_knowledge_derivations(id,space_id,output_node_id,action_kind,action_version,engine,status,created_at,updated_at) VALUES($1,$2,$3,'generate-video',1,$4,'running',$5,$5)")
        .bind(&request.id).bind(&request.space_id).bind(&request.output_node_id)
        .bind(format!("{}:{}", target.provider(), target.model())).bind(now)
        .execute(&mut **tx).await.map_err(db::sql_error)?;
    let values = request
        .inputs
        .iter()
        .map(|input| {
            Ok(inputs::DerivationInput {
                node_id: &input.node_id,
                role: &input.role,
                source_revision: &input.source_revision,
                revision_basis: input_basis(&input.role)?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    inputs::insert(tx, &request.id, &values).await
}

pub(super) fn input_basis(role: &str) -> Result<inputs::RevisionBasis, String> {
    match role {
        "primary" => Ok(inputs::RevisionBasis::Knowledge),
        "source" => Ok(inputs::RevisionBasis::Asset),
        _ => Err("Video generation input role is invalid".into()),
    }
}
