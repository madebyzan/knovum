use std::fs;

use serde_json::Value;
use sha2::{Digest, Sha256};
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use super::{source, CompleteImageCutoutRequest, CompletedDerivationOutput};
use crate::app_error::AppError;
use crate::spaces::derivation::{pending_output, relations};
use crate::spaces::{
    assets::asset_directory,
    db,
    knowledge::{contracts, indexing::revision},
};

pub(super) async fn complete(
    app: AppHandle,
    request: CompleteImageCutoutRequest,
) -> Result<CompletedDerivationOutput, AppError> {
    let path = asset_directory(&app)?.join(format!("derived-{}.png", Uuid::new_v4()));
    let source_path =
        super::super::source_snapshot::image_path(&app, &request.id, &request.source_edit).await?;
    let source_edit = request.source_edit.clone();
    let source = crate::blocking::run({
        let app = app.clone();
        move || render_source(&app, &source_path, &source_edit)
    })
    .await?;
    let cancellation = CancellationToken::new();
    let resources = crate::local_resources::preparation::prepare_models_for_job(
        &app,
        crate::local_resources::preparation::contracts::ResourcePreparationTarget::ImageSegmentation,
        &request.id,
        "Remove image background",
        &cancellation,
        &mut |_| {},
    )
    .await?;
    let deadline_ms = u64::try_from(crate::clock::unix_time_millis()?)
        .map_err(|_| "System clock is before the Unix epoch")?
        .saturating_add(120_000);
    let identity = crate::ai::contracts::image_wire::ImageSegmentationExecutionIdentity {
        owner_kind: "space-derivation".into(),
        owner_id: request.id.clone(),
        attempt_id: request.id.clone(),
        attempt_epoch: 0,
        input_revision: request.source_revision.clone(),
        deadline_ms,
    };
    let cutout =
        crate::images::remove_background(&app, source.png, &identity, &resources, &cancellation)
            .await?;
    let processed = persist_cutout(&path, cutout).await?;
    match persist_completion(&app, &request, path.to_string_lossy().as_ref(), &processed).await {
        Ok(completed) => Ok(completed),
        Err(error) => {
            let _ = fs::remove_file(path);
            Err(error)
        }
    }
}

struct ProcessedCutout {
    digest: String,
    size: usize,
    width: u32,
    height: u32,
}

fn render_source(
    app: &AppHandle,
    source_path: &str,
    source_edit: &Value,
) -> Result<crate::images::EditedImageOutput, AppError> {
    let path =
        crate::spaces::assets::authorize_spaces_asset(app, std::path::Path::new(source_path))?;
    Ok(crate::images::render_source_png(&path, source_edit)?)
}

async fn persist_cutout(
    output: &std::path::Path,
    cutout: crate::images::BackgroundRemovalOutput,
) -> Result<ProcessedCutout, AppError> {
    let digest = format!("sha256:{:x}", Sha256::digest(&cutout.png));
    if let Err(error) = tokio::fs::write(output, &cutout.png).await {
        let _ = tokio::fs::remove_file(output).await;
        return Err(error.to_string().into());
    }
    Ok(ProcessedCutout {
        digest,
        size: cutout.png.len(),
        width: cutout.width,
        height: cutout.height,
    })
}

async fn persist_completion(
    app: &AppHandle,
    request: &CompleteImageCutoutRequest,
    path: &str,
    processed: &ProcessedCutout,
) -> Result<CompletedDerivationOutput, AppError> {
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    let row: Option<(String, String, String, String)> = sqlx::query_as(
        r#"SELECT d.space_id,i.node_id,d.output_node_id,n.data
           FROM spaces_knowledge_derivations d
           JOIN spaces_knowledge_derivation_inputs i ON i.derivation_id=d.id AND i.position=0 AND i.role='primary'
           JOIN spaces_nodes n ON n.id=d.output_node_id
           WHERE d.id=$1 AND d.status='running'"#,
    )
    .bind(&request.id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    let Some((space_id, source_id, output_id, data)) = row else {
        return Err("Background removal is no longer running".into());
    };
    source::revalidate(
        &mut tx,
        &request.id,
        &space_id,
        &source_id,
        &request.source_edit,
    )
    .await?;
    let now = crate::clock::unix_time_millis()?;
    let mut value: Value = serde_json::from_str(&data).map_err(db::json_error)?;
    pending_output::validate_unpublished_asset(&value)?;
    apply_output_content(&mut value, path, processed, now)?;
    contracts::validate_node(&mut tx, &space_id, "image", &value).await?;
    let relation = relations::derives_from(
        &space_id,
        &output_id,
        &source_id,
        "Background removed from source image",
        now,
    );
    crate::spaces::derivation::persist_output_node(&mut tx, &output_id, &value, now).await?;
    relations::insert(&mut tx, &relation).await?;
    sqlx::query("UPDATE spaces_knowledge_derivations SET status='completed',error='',completed_at=$1,updated_at=$1 WHERE id=$2 AND status='running'")
        .bind(now).bind(&request.id).execute(&mut *tx).await.map_err(db::sql_error)?;
    sqlx::query("UPDATE spaces SET updated_at=$1 WHERE id=$2")
        .bind(now)
        .bind(&space_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(CompletedDerivationOutput {
        node_id: output_id,
        data: value,
        relation,
    })
}

fn apply_output_content(
    value: &mut Value,
    path: &str,
    processed: &ProcessedCutout,
    now: i64,
) -> Result<(), AppError> {
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Derived image data is invalid".to_string())?;
    object.insert("assetPath".into(), Value::String(path.into()));
    object.insert(
        "sourceRevision".into(),
        Value::String(processed.digest.clone()),
    );
    object.insert("size".into(), Value::from(processed.size as u64));
    object.insert("modifiedAt".into(), Value::from(now));
    object.insert("intrinsicWidth".into(), Value::from(processed.width));
    object.insert("intrinsicHeight".into(), Value::from(processed.height));
    object.remove("imageEdit");
    object.remove("previewPath");
    object.remove("processingState");
    revision::apply(value)?;
    Ok(())
}
