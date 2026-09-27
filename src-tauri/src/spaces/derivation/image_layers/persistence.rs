use serde_json::Value;
use sqlx::{Sqlite, Transaction};
use tauri::AppHandle;

use super::{assets::PreparedLayer, contracts::GenerateImageLayersRequest, source};
use crate::spaces::{
    db,
    derivation::{inputs, pending_output, relations, CompletedDerivationOutput},
    knowledge::{contracts, indexing::revision},
};

pub async fn start(app: &AppHandle, request: &GenerateImageLayersRequest) -> Result<(), String> {
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    let source_path = source::validate(&mut tx, request).await?;
    for output_id in &request.output_node_ids {
        contracts::validate_same_space_node(
            &mut tx,
            &request.space_id,
            &request.source_node_id,
            output_id,
        )
        .await?;
        let encoded: String = sqlx::query_scalar(
            "SELECT data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND kind='image' AND deleted_at IS NULL",
        )
        .bind(output_id)
        .bind(&request.space_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(db::sql_error)?
        .ok_or("Image layer output node is unavailable")?;
        let output: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
        pending_output::validate_separate_asset(&output, &source_path)?;
    }
    for (index, output_id) in request.output_node_ids.iter().enumerate() {
        let derivation_id = derivation_id(&request.generation_id, index);
        sqlx::query("INSERT INTO spaces_knowledge_derivations(id,space_id,output_node_id,action_kind,action_version,engine,status,created_at,updated_at) VALUES($1,$2,$3,'split-image-layers',1,'fal:qwen-image-layered','running',$4,$4)")
            .bind(&derivation_id).bind(&request.space_id).bind(output_id)
            .bind(crate::clock::unix_time_millis()?).execute(&mut *tx).await.map_err(db::sql_error)?;
        inputs::insert(
            &mut tx,
            &derivation_id,
            &[inputs::DerivationInput {
                node_id: &request.source_node_id,
                role: "primary",
                source_revision: &request.source_revision,
                revision_basis: inputs::RevisionBasis::Rendered,
            }],
        )
        .await?;
    }
    tx.commit().await.map_err(db::sql_error)
}

pub async fn complete(
    app: &AppHandle,
    request: &GenerateImageLayersRequest,
    layers: &[PreparedLayer],
) -> Result<Vec<CompletedDerivationOutput>, String> {
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    source::validate(&mut tx, request).await?;
    let now = crate::clock::unix_time_millis()?;
    let mut completed = Vec::with_capacity(layers.len());
    for (index, (output_id, layer)) in request.output_node_ids.iter().zip(layers).enumerate() {
        let mut data =
            load_pending_output(&mut tx, &derivation_id(&request.generation_id, index)).await?;
        pending_output::validate_unpublished_asset(&data)?;
        apply_layer_content(&mut data, layer, now)?;
        contracts::validate_node(&mut tx, &request.space_id, output_id, "image", &data).await?;
        crate::spaces::derivation::persist_output_node(&mut tx, output_id, &data, now).await?;
        let relation = relations::derives_from(
            &request.space_id,
            output_id,
            &request.source_node_id,
            "Image layer separated from source image",
            now,
        );
        relations::insert(&mut tx, &relation).await?;
        sqlx::query("UPDATE spaces_knowledge_derivations SET status='completed',completed_at=$1,updated_at=$1 WHERE id=$2 AND status='running'")
            .bind(now).bind(derivation_id(&request.generation_id, index)).execute(&mut *tx).await.map_err(db::sql_error)?;
        completed.push(CompletedDerivationOutput {
            node_id: output_id.clone(),
            data,
            relation,
        });
    }
    sqlx::query("UPDATE spaces SET updated_at=$1 WHERE id=$2")
        .bind(now)
        .bind(&request.space_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(completed)
}

pub async fn fail(app: &AppHandle, request: &GenerateImageLayersRequest, error: &str) {
    for index in 0..request.output_node_ids.len() {
        let _ = super::super::failure::fail_message(
            app,
            &derivation_id(&request.generation_id, index),
            error,
        )
        .await;
    }
}

async fn load_pending_output(
    tx: &mut Transaction<'_, Sqlite>,
    derivation_id: &str,
) -> Result<Value, String> {
    let encoded = sqlx::query_scalar::<_, String>(
        "SELECT n.data FROM spaces_knowledge_derivations d JOIN spaces_nodes n ON n.id=d.output_node_id WHERE d.id=$1 AND d.status='running' AND n.kind='image' AND n.deleted_at IS NULL",
    )
    .bind(derivation_id).fetch_optional(&mut **tx).await.map_err(db::sql_error)?
    .ok_or("Image layer output is no longer pending")?;
    serde_json::from_str(&encoded).map_err(db::json_error)
}

fn apply_layer_content(data: &mut Value, layer: &PreparedLayer, now: i64) -> Result<(), String> {
    let object = data
        .as_object_mut()
        .ok_or("Image layer output data is invalid")?;
    object.insert(
        "assetPath".into(),
        layer.path.to_string_lossy().into_owned().into(),
    );
    object.insert("sourceRevision".into(), layer.digest.clone().into());
    object.insert("size".into(), (layer.size as u64).into());
    object.insert("modifiedAt".into(), now.into());
    object.insert("intrinsicWidth".into(), layer.width.into());
    object.insert("intrinsicHeight".into(), layer.height.into());
    for field in [
        "processingState",
        "imageEdit",
        "previewPath",
        "visualKnowledge",
    ] {
        object.remove(field);
    }
    revision::apply(data)?;
    Ok(())
}

pub fn derivation_id(group_id: &str, index: usize) -> String {
    format!("{group_id}:layer-{index}")
}
