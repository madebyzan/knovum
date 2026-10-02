use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tauri::AppHandle;

use super::{inputs, pending_output, relations, source_validation};
use crate::app_error::AppError;
use crate::spaces::{
    assets::asset_directory,
    db,
    knowledge::{contracts, indexing::revision},
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct DocumentPublication {
    id: String,
    space_id: String,
    output_node_id: String,
    inputs: Vec<DocumentInput>,
    #[serde(skip)]
    bytes: Vec<u8>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DocumentInput {
    node_id: String,
    role: String,
    source_revision: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentOutput {
    node_id: String,
    data: Value,
}

pub(super) async fn start(
    app: &AppHandle,
    prepared: Value,
    action_id: &str,
    engine: &str,
    authority: &super::action_execution::ExecutionAuthority,
) -> Result<(), AppError> {
    let request: DocumentPublication = serde_json::from_value(prepared).map_err(db::json_error)?;
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    authority.require_active(&mut *tx).await?;
    let now = crate::clock::unix_time_millis()?;
    sqlx::query("INSERT INTO spaces_knowledge_derivations(id,space_id,output_node_id,action_kind,action_version,engine,status,created_at,updated_at) VALUES($1,$2,$3,$4,1,$5,'running',$6,$6)")
        .bind(&request.id).bind(&request.space_id).bind(&request.output_node_id).bind(action_id).bind(engine).bind(now).execute(&mut *tx).await.map_err(db::sql_error)?;
    let sources = request
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
        .collect::<Result<Vec<_>, AppError>>()?;
    inputs::insert(&mut tx, &request.id, &sources).await?;
    tx.commit()
        .await
        .map_err(db::sql_error)
        .map_err(AppError::from)
}

pub(super) async fn publish(
    app: &AppHandle,
    prepared_action: Value,
    bytes: Vec<u8>,
    action_id: &str,
    engine: &str,
    authority: &super::action_execution::ExecutionAuthority,
) -> Result<DocumentOutput, AppError> {
    // Prepared Action metadata contains the resolved source document.
    // Rendered bytes come only from the Rust document engine, never from Renderer input.
    let mut request: DocumentPublication =
        serde_json::from_value(prepared_action).map_err(db::json_error)?;
    request.bytes = bytes;
    validate(&request)?;
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
            input_basis(&input.role)?,
        )
        .await?;
    }
    let encoded: String = sqlx::query_scalar(
        "SELECT data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND kind='file'",
    )
    .bind(&request.output_node_id)
    .bind(&request.space_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    let mut data: Value = serde_json::from_str(&encoded)
        .map_err(db::json_error)
        .map_err(AppError::from)?;
    let path = pending_output::validate_unpublished_asset(&data)?.to_owned();
    let parent = std::path::Path::new(&path)
        .parent()
        .ok_or("Document output path is invalid")?
        .canonicalize()
        .map_err(|error| format!("Unable to resolve output directory: {error}"))?;
    let expected_dir = asset_directory(app)?
        .canonicalize()
        .map_err(|error| format!("Unable to resolve asset directory: {error}"))?;
    if parent != expected_dir {
        return Err("Document output path is outside the Space asset directory".into());
    }
    let file = super::scratch_file::ScratchFile::new(std::path::PathBuf::from(&path));
    tokio::fs::write(&path, &request.bytes)
        .await
        .map_err(|error| error.to_string())?;
    persist(&mut tx, &request, &mut data, action_id, engine).await?;
    authority.require_active(&mut *tx).await?;
    tx.commit()
        .await
        .map_err(db::sql_error)
        .map_err(AppError::from)?;
    file.retain();
    Ok(DocumentOutput {
        node_id: request.output_node_id,
        data,
    })
}

async fn persist(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    request: &DocumentPublication,
    data: &mut Value,
    action_id: &str,
    engine: &str,
) -> Result<(), AppError> {
    let now = crate::clock::unix_time_millis()?;
    let object = data
        .as_object_mut()
        .ok_or("Document node data is invalid")?;
    object.insert(
        "sourceRevision".into(),
        format!("sha256:{:x}", Sha256::digest(&request.bytes)).into(),
    );
    object.insert("size".into(), (request.bytes.len() as u64).into());
    object.insert("modifiedAt".into(), now.into());
    object.remove("processingState");
    revision::apply(data)?;
    contracts::validate_node(tx, &request.space_id, "file", data).await?;
    super::persist_output_node(tx, &request.output_node_id, data, now).await?;
    let changed = sqlx::query("UPDATE spaces_knowledge_derivations SET status='completed',updated_at=$1,completed_at=$1 WHERE id=$2 AND status='running' AND action_kind=$3 AND engine=$4")
        .bind(now).bind(&request.id).bind(action_id).bind(engine).execute(&mut **tx).await.map_err(db::sql_error)?;
    if changed.rows_affected() != 1 {
        return Err("Document derivation is no longer running".into());
    }
    inputs::validate_current(tx, &request.id).await?;
    for input in &request.inputs {
        relations::insert(
            tx,
            &relations::derives_from(
                &request.space_id,
                &request.output_node_id,
                &input.node_id,
                "Document exported from current Markdown",
                now,
            ),
        )
        .await?;
    }
    Ok(())
}

fn input_basis(role: &str) -> Result<inputs::RevisionBasis, AppError> {
    match role {
        "primary" => Ok(inputs::RevisionBasis::Knowledge),
        "source" => Ok(inputs::RevisionBasis::Asset),
        _ => Err("Document input role is invalid".into()),
    }
}

fn validate(request: &DocumentPublication) -> Result<(), AppError> {
    if request.bytes.is_empty()
        || request.inputs.len() != 1
        || request
            .inputs
            .first()
            .is_none_or(|input| input.role != "primary")
    {
        return Err("Document request is invalid".into());
    }
    Ok(())
}
