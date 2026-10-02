use std::path::Path;

use serde_json::Value;
use tauri::AppHandle;

use super::GenerateVideoRequest;
use crate::ai::tasks::TaskExecutionTarget;
use crate::spaces::derivation::{inputs, pending_output, source_validation};
use crate::spaces::{assets::asset_directory, db, knowledge::contracts};

pub struct PreparedVideo {
    pub path: String,
    pub reference_images: Vec<String>,
    pub provider_prompt: String,
    // Own temporary rendered references until the provider/Agent finishes consuming them.
    pub _reference_files: Vec<super::super::scratch_file::ScratchFile>,
}

pub async fn start(
    app: &AppHandle,
    request: &GenerateVideoRequest,
    target: &TaskExecutionTarget,
    authority: &super::super::action_execution::ExecutionAuthority,
) -> Result<PreparedVideo, String> {
    validate_request(request)?;
    super::validation::require_decoder(app)?;
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
    let context = load_context(&mut tx, request).await?;
    let provider_prompt = super::prompt::assemble(&context, &request.prompt)?;
    let (references, reference_files) = load_references(&mut tx, request, &asset_directory).await?;
    let path = output(&mut tx, request, &asset_directory).await?;
    persist_start(&mut tx, request, target).await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(PreparedVideo {
        path,
        reference_images: references,
        provider_prompt,
        _reference_files: reference_files,
    })
}

fn validate_request(request: &GenerateVideoRequest) -> Result<(), String> {
    if !(5..=15).contains(&request.duration_seconds)
        || request.prompt.trim().is_empty()
        || request.prompt.encode_utf16().count() > 16_000
        || request.inputs.is_empty()
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

async fn load_context(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    request: &GenerateVideoRequest,
) -> Result<String, String> {
    let mut context = String::new();
    for input in &request.inputs {
        let encoded: String = sqlx::query_scalar(
            "SELECT data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND deleted_at IS NULL",
        )
        .bind(&input.node_id)
        .bind(&request.space_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(db::sql_error)?;
        let data: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
        let chunks: Vec<String> = sqlx::query_scalar("SELECT text FROM spaces_current_knowledge_chunks WHERE node_id=$1 AND space_id=$2 ORDER BY created_at,id")
            .bind(&input.node_id).bind(&request.space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?;
        if chunks.is_empty() && data["kind"] != "image" {
            return Err("Video generation requires current knowledge or an image reference for every selected source".into());
        }
        context.push_str(&format!(
            "\nSOURCE {}\n",
            data["title"]
                .as_str()
                .ok_or("Video source title is invalid")?
        ));
        for text in chunks {
            context.push_str(&text);
            context.push('\n');
        }
        if context.len() > 128 * 1024 {
            return Err("Video source context exceeds 128 KiB; select a smaller source set".into());
        }
    }
    Ok(context)
}

async fn load_references(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    request: &GenerateVideoRequest,
    directory: &Path,
) -> Result<(Vec<String>, Vec<super::super::scratch_file::ScratchFile>), String> {
    let mut files = Vec::new();
    let mut values = Vec::with_capacity(request.inputs.len().saturating_sub(1));
    for input in &request.inputs {
        let encoded: String = sqlx::query_scalar(
            "SELECT data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND deleted_at IS NULL",
        )
        .bind(&input.node_id)
        .bind(&request.space_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(db::sql_error)?
        .ok_or("Video reference must be a current image")?;
        let data: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
        if data["kind"] != "image" {
            continue;
        }
        if values.len() == 10 {
            return Err("Video generation accepts at most 10 reference images".into());
        }
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
        let revision = data["sourceRevision"]
            .as_str()
            .ok_or("Video reference source revision is unavailable")?;
        crate::file_inspection::verify_content_revision(path, revision)?;
        let source = path.to_owned();
        let edit = data.get("imageEdit").cloned().unwrap_or(Value::Null);
        let rendered =
            crate::blocking::run(move || crate::images::render_source_png(&source, &edit)).await?;
        crate::file_inspection::verify_content_revision(path, revision)?;
        let reference = directory.join(format!("video-reference-{}.png", uuid::Uuid::new_v4()));
        let file = super::super::scratch_file::ScratchFile::new(reference.clone());
        tokio::fs::write(&reference, &rendered.png)
            .await
            .map_err(|error| error.to_string())?;
        files.push(file);
        values.push(reference.to_string_lossy().into_owned());
    }
    Ok((values, files))
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
    target: &TaskExecutionTarget,
) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    sqlx::query("INSERT INTO spaces_knowledge_derivations(id,space_id,output_node_id,action_kind,action_version,engine,status,created_at,updated_at) VALUES($1,$2,$3,'generate-video',1,$4,'running',$5,$5)")
        .bind(&request.id).bind(&request.space_id).bind(&request.output_node_id)
        .bind(target.engine()).bind(now)
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
        "primary" | "source" => Ok(inputs::RevisionBasis::Knowledge),
        _ => Err("Video generation input role is invalid".into()),
    }
}
