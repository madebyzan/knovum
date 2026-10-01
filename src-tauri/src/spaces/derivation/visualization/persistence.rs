use std::path::Path;

use serde_json::Value;
use sha2::{Digest, Sha256};
use tauri::AppHandle;

use super::{validation, GenerateVisualizationRequest, VisualizationOutput, VisualizationResult};
use crate::spaces::derivation::{inputs, pending_output, relations, source_validation};
use crate::{
    ai::tasks::TaskExecutionTarget,
    spaces::{
        assets::asset_directory,
        db,
        knowledge::{contracts, indexing::revision},
    },
};

pub async fn start(
    app: &AppHandle,
    request: &GenerateVisualizationRequest,
    target: &TaskExecutionTarget,
    authority: &super::super::action_execution::ExecutionAuthority,
) -> Result<(), String> {
    validation::validate_request(request)?;
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    authority.require_active(&mut *tx).await?;
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
            inputs::RevisionBasis::Knowledge,
        )
        .await?;
    }
    let output: String = sqlx::query_scalar("SELECT data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND kind='image' AND deleted_at IS NULL")
        .bind(&request.output_node_id).bind(&request.space_id).fetch_optional(&mut *tx).await.map_err(db::sql_error)?
        .ok_or("Visualization output node is unavailable")?;
    let output: Value = serde_json::from_str(&output).map_err(db::json_error)?;
    let path = pending_output::validate_unpublished_asset(&output)?;
    if Path::new(path).parent() != Some(asset_directory(app)?.as_path()) {
        return Err("Visualization output path is outside the Space asset directory".into());
    }
    let now = crate::clock::unix_time_millis()?;
    sqlx::query("INSERT INTO spaces_knowledge_derivations(id,space_id,output_node_id,action_kind,action_version,engine,status,created_at,updated_at) VALUES($1,$2,$3,(SELECT json_extract(input_snapshot,'$.actionId') FROM spaces_runs WHERE id=$1 AND kind='action'),1,$4,'running',$5,$5)")
        .bind(&request.id).bind(&request.space_id).bind(&request.output_node_id)
        .bind(target.engine()).bind(now)
        .execute(&mut *tx).await.map_err(db::sql_error)?;
    let values = request
        .inputs
        .iter()
        .map(|input| inputs::DerivationInput {
            node_id: &input.node_id,
            role: &input.role,
            source_revision: &input.source_revision,
            revision_basis: inputs::RevisionBasis::Knowledge,
        })
        .collect::<Vec<_>>();
    inputs::insert(&mut tx, &request.id, &values).await?;
    tx.commit().await.map_err(db::sql_error)
}

pub async fn complete(
    app: &AppHandle,
    request: &GenerateVisualizationRequest,
    generated: VisualizationResult,
    authority: &super::super::action_execution::ExecutionAuthority,
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<VisualizationOutput, String> {
    let (bytes, width, height) = validation::canonical_png(&generated.bytes)?;
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    authority.require_active(&mut *tx).await?;
    let encoded: String = sqlx::query_scalar("SELECT n.data FROM spaces_knowledge_derivations d JOIN spaces_nodes n ON n.id=d.output_node_id WHERE d.id=$1 AND d.status='running'")
        .bind(&request.id).fetch_optional(&mut *tx).await.map_err(db::sql_error)?.ok_or("Visualization is no longer running")?;
    for input in &request.inputs {
        source_validation::validate(
            &mut tx,
            &request.space_id,
            &input.node_id,
            &input.source_revision,
            inputs::RevisionBasis::Knowledge,
        )
        .await?;
    }
    let mut data: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
    let path = pending_output::validate_unpublished_asset(&data)?.to_owned();
    tokio::fs::write(&path, &bytes)
        .await
        .map_err(|error| error.to_string())?;
    let persisted = persist_completion(
        &mut tx, request, &generated, &mut data, &bytes, width, height,
    )
    .await;
    match persisted {
        Ok(relations) => {
            if cancellation.is_cancelled() {
                let _ = tokio::fs::remove_file(&path).await;
                return Err("Visualization generation was cancelled".into());
            }
            tx.commit().await.map_err(db::sql_error)?;
            Ok(VisualizationOutput {
                node_id: request.output_node_id.clone(),
                data,
                relations,
            })
        }
        Err(error) => {
            let _ = tokio::fs::remove_file(path).await;
            Err(error)
        }
    }
}

pub async fn fail(app: &AppHandle, id: &str, error: &str) -> Result<(), String> {
    crate::spaces::derivation::failure::fail_message(app, id, error).await
}

async fn persist_completion(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    request: &GenerateVisualizationRequest,
    generated: &VisualizationResult,
    data: &mut Value,
    bytes: &[u8],
    width: u32,
    height: u32,
) -> Result<Vec<super::super::DerivationRelation>, String> {
    let now = crate::clock::unix_time_millis()?;
    let object = data
        .as_object_mut()
        .ok_or("Visualization node data is invalid")?;
    object.insert(
        "sourceRevision".into(),
        format!("sha256:{:x}", Sha256::digest(bytes)).into(),
    );
    object.insert("size".into(), (bytes.len() as u64).into());
    object.insert("modifiedAt".into(), now.into());
    object.insert("intrinsicWidth".into(), width.into());
    object.insert("intrinsicHeight".into(), height.into());
    object.remove("processingState");
    revision::apply(data)?;
    contracts::validate_node(tx, &request.space_id, "image", data).await?;
    crate::spaces::derivation::persist_output_node(tx, &request.output_node_id, data, now).await?;
    let mut output_relations = Vec::with_capacity(request.inputs.len());
    for input in &request.inputs {
        let relation = relations::derives_from(
            &request.space_id,
            &request.output_node_id,
            &input.node_id,
            "Visualization generated from selected knowledge",
            now,
        );
        relations::insert(tx, &relation).await?;
        output_relations.push(relation);
    }
    sqlx::query("INSERT INTO spaces_generated_images(derivation_id,intent,style,aspect_ratio,execution_source,revised_prompt) VALUES($1,$2,$3,$4,$5,$6)")
        .bind(&request.id).bind(&request.intent).bind(&request.style).bind(&request.aspect_ratio)
        .bind(serde_json::to_string(&generated.execution_source).map_err(db::json_error)?).bind(&generated.revised_prompt)
        .execute(&mut **tx).await.map_err(db::sql_error)?;
    sqlx::query("UPDATE spaces_knowledge_derivations SET status='completed',completed_at=$1,updated_at=$1 WHERE id=$2")
        .bind(now).bind(&request.id).execute(&mut **tx).await.map_err(db::sql_error)?;
    Ok(output_relations)
}
