use std::path::Path;

use serde_json::Value;
use tauri::AppHandle;

use super::{contracts::DeriveImageRequest, output::CanonicalImage, output_file, validation};
use crate::spaces::{
    db,
    derivation::{inputs, pending_output, relations, source_validation, CompletedDerivationOutput},
    knowledge::{contracts, indexing::revision},
};

pub async fn start(app: &AppHandle, request: &DeriveImageRequest) -> Result<(), String> {
    validation::request(request)?;
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    contracts::validate_same_space_node(
        &mut tx,
        &request.space_id,
        &request.source_node_id,
        &request.output_node_id,
    )
    .await?;
    let source_kind: Option<String> = sqlx::query_scalar(
        "SELECT kind FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND deleted_at IS NULL",
    )
    .bind(&request.source_node_id)
    .bind(&request.space_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    if source_kind.as_deref() != Some("image") {
        return Err("Image operation source is unavailable".into());
    }
    source_validation::validate(
        &mut tx,
        &request.space_id,
        &request.source_node_id,
        &request.source_revision,
        inputs::RevisionBasis::Asset,
    )
    .await?;
    let source_path = validation::source_edit(&mut tx, request).await?;
    let output: String = sqlx::query_scalar("SELECT data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND kind='image' AND deleted_at IS NULL")
        .bind(&request.output_node_id).bind(&request.space_id).fetch_optional(&mut *tx).await.map_err(db::sql_error)?
        .ok_or("Image operation output node is unavailable")?;
    let output: Value = serde_json::from_str(&output).map_err(db::json_error)?;
    pending_output::validate_separate_asset(&output, &source_path)?;
    let now = crate::clock::unix_time_millis()?;
    sqlx::query("INSERT INTO spaces_knowledge_derivations(id,space_id,output_node_id,action_kind,action_version,engine,status,created_at,updated_at) VALUES($1,$2,$3,$4,1,$5,'running',$6,$6)")
        .bind(&request.id).bind(&request.space_id).bind(&request.output_node_id)
        .bind(request.operation.action_kind()).bind(request.operation.engine())
        .bind(now).execute(&mut *tx).await.map_err(db::sql_error)?;
    inputs::insert(
        &mut tx,
        &request.id,
        &[inputs::DerivationInput {
            node_id: &request.source_node_id,
            role: "primary",
            source_revision: &request.source_revision,
            revision_basis: inputs::RevisionBasis::Rendered,
        }],
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)
}

pub async fn complete(
    app: &AppHandle,
    request: &DeriveImageRequest,
    image: CanonicalImage,
) -> Result<CompletedDerivationOutput, String> {
    let path = output_file::write(app, &image.png).await?;
    let completed = complete_persistence(app, request, &image, &path).await;
    if completed.is_err() {
        let _ = tokio::fs::remove_file(&path).await;
    }
    completed
}

async fn complete_persistence(
    app: &AppHandle,
    request: &DeriveImageRequest,
    image: &CanonicalImage,
    path: &Path,
) -> Result<CompletedDerivationOutput, String> {
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    source_validation::validate(
        &mut tx,
        &request.space_id,
        &request.source_node_id,
        &request.source_revision,
        inputs::RevisionBasis::Asset,
    )
    .await?;
    validation::source_edit(&mut tx, request).await?;
    let encoded: String = sqlx::query_scalar("SELECT n.data FROM spaces_knowledge_derivations d JOIN spaces_nodes n ON n.id=d.output_node_id AND n.space_id=d.space_id WHERE d.id=$1 AND d.status='running' AND n.kind='image' AND n.deleted_at IS NULL")
        .bind(&request.id).fetch_optional(&mut *tx).await.map_err(db::sql_error)?
        .ok_or("Image operation is no longer running")?;
    let mut data: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
    pending_output::validate_unpublished_asset(&data)?;
    let relation = persist(&mut tx, request, &mut data, image, path).await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(CompletedDerivationOutput {
        node_id: request.output_node_id.clone(),
        data,
        relation,
    })
}

async fn persist(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    request: &DeriveImageRequest,
    data: &mut Value,
    image: &CanonicalImage,
    path: &Path,
) -> Result<super::super::DerivationRelation, String> {
    let now = crate::clock::unix_time_millis()?;
    let object = data
        .as_object_mut()
        .ok_or("Image operation node data is invalid")?;
    object.insert(
        "assetPath".into(),
        path.to_string_lossy().into_owned().into(),
    );
    object.insert("sourceRevision".into(), image.digest.clone().into());
    object.insert("size".into(), (image.png.len() as u64).into());
    object.insert("modifiedAt".into(), now.into());
    object.insert("intrinsicWidth".into(), image.width.into());
    object.insert("intrinsicHeight".into(), image.height.into());
    object.remove("processingState");
    object.remove("imageEdit");
    object.remove("previewPath");
    revision::apply(data)?;
    contracts::validate_node(
        tx,
        &request.space_id,
        &request.output_node_id,
        "image",
        data,
    )
    .await?;
    crate::spaces::derivation::persist_output_node(tx, &request.output_node_id, data, now).await?;
    let relation = relations::derives_from(
        &request.space_id,
        &request.output_node_id,
        &request.source_node_id,
        request.operation.rationale(),
        now,
    );
    relations::insert(tx, &relation).await?;
    sqlx::query("UPDATE spaces_knowledge_derivations SET status='completed',completed_at=$1,updated_at=$1 WHERE id=$2 AND status='running'")
        .bind(now).bind(&request.id).execute(&mut **tx).await.map_err(db::sql_error)?;
    sqlx::query("UPDATE spaces SET updated_at=$1 WHERE id=$2")
        .bind(now)
        .bind(&request.space_id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    Ok(relation)
}
