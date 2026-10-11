use tauri::AppHandle;

use super::GenerateGroundedDocumentRequest;
use crate::spaces::db;
use crate::spaces::derivation::inputs;
use crate::spaces::derivation::source_validation;

pub async fn start(
    app: &AppHandle,
    request: &GenerateGroundedDocumentRequest,
    authority: &super::super::action_execution::ExecutionAuthority,
) -> Result<(), String> {
    validate_request(request)?;
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    authority.require_active(&mut *tx).await?;
    for input in &request.inputs {
        crate::spaces::knowledge::validation::validate_same_space_node(
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
    let now = crate::foundation::clock::unix_time_millis()?;
    sqlx::query(
        "INSERT INTO spaces_knowledge_derivations(id,space_id,output_node_id,action_kind,action_version,engine,status,created_at,updated_at) VALUES($1,$2,$3,(SELECT json_extract(input_snapshot,'$.actionId') FROM spaces_runs WHERE id=$1 AND kind='action'),1,'knovum-grounded-ai:1','running',$4,$4)",
    )
    .bind(&request.id)
    .bind(&request.space_id)
    .bind(&request.output_node_id)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
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

pub async fn fail(app: &AppHandle, derivation_id: &str, message: &str) -> Result<(), String> {
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    let data: Option<String> = sqlx::query_scalar(
        "SELECT n.data FROM spaces_knowledge_derivations d JOIN spaces_nodes n ON n.id=d.output_node_id WHERE d.id=$1 AND d.status='running'",
    )
    .bind(derivation_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    let Some(data) = data else { return Ok(()) };
    let now = crate::foundation::clock::unix_time_millis()?;
    let mut value: serde_json::Value = serde_json::from_str(&data).map_err(db::json_error)?;
    value["derivationState"] = "failed".into();
    sqlx::query("UPDATE spaces_nodes SET data=$1,updated_at=$2 WHERE id=(SELECT output_node_id FROM spaces_knowledge_derivations WHERE id=$3)")
        .bind(sqlx::types::Json(&value))
        .bind(now).bind(derivation_id).execute(&mut *tx).await.map_err(db::sql_error)?;
    sqlx::query("UPDATE spaces_knowledge_derivations SET status='failed',error=$1,completed_at=$2,updated_at=$2 WHERE id=$3 AND status='running'")
        .bind(message).bind(now).bind(derivation_id).execute(&mut *tx).await.map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)
}

fn validate_request(request: &GenerateGroundedDocumentRequest) -> Result<(), String> {
    if request.id.trim().is_empty()
        || request.space_id.trim().is_empty()
        || request.output_node_id.trim().is_empty()
        || request.purpose.trim().is_empty()
        || request.inputs.is_empty()
        || request
            .inputs
            .first()
            .is_none_or(|input| input.role != "primary")
        || request
            .inputs
            .iter()
            .filter(|input| input.role == "primary")
            .count()
            != 1
    {
        return Err("Grounded document request is invalid".into());
    }
    Ok(())
}
