use serde_json::Value;
use tauri::AppHandle;

use super::{GenerateGroundedDocumentRequest, GeneratedDocument, GroundedDocumentOutput};
use crate::foundation::app_error::AppError;
use crate::spaces::derivation::relations;
use crate::spaces::derivation::source_inputs;
use crate::spaces::{db, knowledge::model::revision};

pub async fn complete(
    app: &AppHandle,
    request: &GenerateGroundedDocumentRequest,
    generated: GeneratedDocument,
    authority: &super::super::action_execution::ExecutionAuthority,
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<GroundedDocumentOutput, AppError> {
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    authority.require_active(&mut *tx).await?;
    let data: String = sqlx::query_scalar("SELECT n.data FROM spaces_knowledge_derivations d JOIN spaces_nodes n ON n.id=d.output_node_id WHERE d.id=$1 AND d.status='running'")
        .bind(&request.id).fetch_optional(&mut *tx).await.map_err(db::sql_error)?
        .ok_or("Grounded document derivation is no longer running")?;
    source_inputs::pin(&mut tx, &request.space_id, &request.inputs).await?;
    let now = crate::foundation::clock::unix_time_millis()?;
    let mut data: Value = serde_json::from_str(&data).map_err(db::json_error)?;
    apply_document(&mut data, &generated.text)?;
    crate::spaces::knowledge::validation::validate_node(
        &mut tx,
        &request.space_id,
        "markdown",
        &data,
    )
    .await?;
    crate::spaces::derivation::persist_output_node(&mut tx, &request.output_node_id, &data, now)
        .await?;
    let output_relations = create_relations(&mut tx, request, now).await?;
    persist_generation(&mut tx, &request.id, &generated).await?;
    sqlx::query("UPDATE spaces_knowledge_derivations SET status='completed',completed_at=$1,updated_at=$1 WHERE id=$2")
        .bind(now).bind(&request.id).execute(&mut *tx).await.map_err(db::sql_error)?;
    if cancellation.is_cancelled() {
        return Err("Grounded document creation was cancelled".into());
    }
    tx.commit().await.map_err(db::sql_error)?;
    Ok(GroundedDocumentOutput {
        node_id: request.output_node_id.clone(),
        data,
        relations: output_relations,
    })
}

fn apply_document(payload: &mut Value, text: &str) -> Result<(), AppError> {
    let text = text.trim();
    if text.is_empty() {
        return Err("Generated Markdown is empty".into());
    }
    let object = payload
        .as_object_mut()
        .ok_or("Pending Markdown data is invalid")?;
    object.insert("text".into(), text.into());
    object.remove("derivationState");
    revision::apply(payload).map(|_| ()).map_err(AppError::from)
}

async fn create_relations(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    request: &GenerateGroundedDocumentRequest,
    now: i64,
) -> Result<Vec<crate::spaces::derivation::DerivationRelation>, AppError> {
    let mut output = Vec::with_capacity(request.inputs.len());
    for input in &request.inputs {
        let relation = relations::derives_from(
            &request.space_id,
            &request.output_node_id,
            &input.node_id,
            "Grounded document drafted from selected knowledge",
            now,
        );
        relations::insert(tx, &relation).await?;
        output.push(relation);
    }
    Ok(output)
}

async fn persist_generation(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    derivation_id: &str,
    generated: &GeneratedDocument,
) -> Result<(), AppError> {
    sqlx::query("INSERT INTO spaces_grounded_documents(derivation_id,citations,execution_source) VALUES($1,$2,$3)")
        .bind(derivation_id).bind(sqlx::types::Json(&generated.citations)).bind(sqlx::types::Json(&generated.execution_source))
        .execute(&mut **tx).await.map_err(db::sql_error)?;
    Ok(())
}
