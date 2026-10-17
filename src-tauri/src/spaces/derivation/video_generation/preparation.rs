use std::path::Path;

use serde_json::Value;
use tauri::AppHandle;

use super::GenerateVideoRequest;
use crate::ai::tasks::TaskExecutionTarget;
use crate::foundation::scratch_file::ScratchFile;
use crate::spaces::derivation::{inputs, pending_output, source_inputs};
use crate::spaces::{assets::asset_directory, db};

/// Video providers read references from disk, so the rendered pixels are kept
/// as files the Run owns until the provider is done with them.
const MAX_REFERENCE_BYTES: usize = 32 * 1024 * 1024;
const MAX_REFERENCES: usize = 10;

pub struct PreparedVideo {
    pub path: String,
    pub reference_images: Vec<String>,
    pub provider_prompt: String,
    // Own temporary rendered references until the provider/Agent finishes consuming them.
    pub _reference_files: Vec<ScratchFile>,
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
    let asset_directory = asset_directory(app)?;
    let (references, reference_files) = write_references(pool, request, &asset_directory).await?;
    let mut tx = db::begin_write(pool).await?;
    authority.require_active(&mut *tx).await?;
    source_inputs::pin_for_output(
        &mut tx,
        &request.space_id,
        &request.output_node_id,
        &request.inputs,
    )
    .await?;
    let context = load_context(&mut tx, request).await?;
    let provider_prompt = super::prompt::assemble(&context, &request.prompt)?;
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
        || request
            .inputs
            .iter()
            .skip(1)
            .any(|input| input.role != "source")
    {
        return Err("Video generation inputs are invalid".into());
    }
    source_inputs::require_single_primary(&request.inputs)
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

async fn write_references(
    pool: &sqlx::SqlitePool,
    request: &GenerateVideoRequest,
    directory: &Path,
) -> Result<(Vec<String>, Vec<ScratchFile>), String> {
    let rendered = source_inputs::render_images(
        pool,
        &request.space_id,
        &request.inputs,
        directory,
        MAX_REFERENCE_BYTES,
    )
    .await?;
    if rendered.len() > MAX_REFERENCES {
        return Err(format!(
            "Video generation accepts at most {MAX_REFERENCES} reference images"
        ));
    }
    let (paths, files) =
        source_inputs::write_png_files(&rendered, directory, "video-reference").await?;
    Ok((
        paths
            .iter()
            .map(|path| path.to_string_lossy().into_owned())
            .collect(),
        files,
    ))
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
    let now = crate::foundation::clock::unix_time_millis()?;
    sqlx::query("INSERT INTO spaces_knowledge_derivations(id,space_id,output_node_id,action_kind,action_version,engine,status,created_at,updated_at) VALUES($1,$2,$3,'generate-video',1,$4,'running',$5,$5)")
        .bind(&request.id).bind(&request.space_id).bind(&request.output_node_id)
        .bind(target.engine()).bind(now)
        .execute(&mut **tx).await.map_err(db::sql_error)?;
    inputs::insert(tx, &request.id, &source_inputs::records(&request.inputs)).await
}
