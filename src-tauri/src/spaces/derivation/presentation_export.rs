use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tauri::AppHandle;

use super::{inputs, relations, source_validation};
use crate::spaces::{
    assets::asset_directory,
    db,
    knowledge::{contracts, indexing::revision},
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompletePresentationRequest {
    id: String,
    space_id: String,
    output_node_id: String,
    inputs: Vec<PresentationInput>,
    bytes: Vec<u8>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PresentationInput {
    node_id: String,
    role: String,
    source_revision: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresentationOutput {
    node_id: String,
    data: Value,
}

pub async fn complete(
    app: &AppHandle,
    request: CompletePresentationRequest,
) -> Result<PresentationOutput, String> {
    validate(&request)?;
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    for input in &request.inputs {
        contracts::validate_same_space_node(
            &mut tx,
            &request.space_id,
            &input.node_id,
            &request.output_node_id,
        )
        .await?;
        source_validation::validate(&mut tx, &input.node_id, &input.source_revision).await?;
    }
    let encoded: String = sqlx::query_scalar(
        "SELECT data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND kind='file'",
    )
    .bind(&request.output_node_id)
    .bind(&request.space_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    let mut data: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
    let path = data
        .get("assetPath")
        .and_then(Value::as_str)
        .ok_or("Presentation output path is unavailable")?
        .to_owned();
    let parent = std::path::Path::new(&path)
        .parent()
        .ok_or("Presentation output path is invalid")?
        .canonicalize()
        .map_err(|error| format!("Unable to resolve output directory: {error}"))?;
    let expected_dir = asset_directory(app)?
        .canonicalize()
        .map_err(|error| format!("Unable to resolve asset directory: {error}"))?;
    if parent != expected_dir {
        return Err("Presentation output path is outside the Space asset directory".into());
    }
    tokio::fs::write(&path, &request.bytes)
        .await
        .map_err(|error| error.to_string())?;
    if let Err(error) = persist(&mut tx, &request, &mut data).await {
        let _ = tokio::fs::remove_file(path).await;
        return Err(error);
    }
    tx.commit().await.map_err(db::sql_error)?;
    Ok(PresentationOutput {
        node_id: request.output_node_id,
        data,
    })
}

async fn persist(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    request: &CompletePresentationRequest,
    data: &mut Value,
) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    let object = data
        .as_object_mut()
        .ok_or("Presentation node data is invalid")?;
    object.insert(
        "sourceRevision".into(),
        format!("sha256:{:x}", Sha256::digest(&request.bytes)).into(),
    );
    object.insert("size".into(), (request.bytes.len() as u64).into());
    object.insert("modifiedAt".into(), now.into());
    object.remove("processingState");
    revision::apply(data)?;
    contracts::validate_node(tx, &request.space_id, &request.output_node_id, "file", data).await?;
    super::persist_output_node(tx, &request.output_node_id, data, now).await?;
    sqlx::query("INSERT INTO spaces_knowledge_derivations(id,space_id,output_node_id,action_kind,action_version,engine,status,created_at,updated_at,completed_at) VALUES($1,$2,$3,'knowledge-presentation',1,'pptxgenjs:4.0.1','completed',$4,$4,$4)")
        .bind(&request.id).bind(&request.space_id).bind(&request.output_node_id).bind(now).execute(&mut **tx).await.map_err(db::sql_error)?;
    let values = request
        .inputs
        .iter()
        .map(|input| inputs::DerivationInput {
            node_id: &input.node_id,
            role: &input.role,
            source_revision: &input.source_revision,
        })
        .collect::<Vec<_>>();
    inputs::insert(tx, &request.id, &values).await?;
    for input in &request.inputs {
        relations::insert(
            tx,
            &relations::derives_from(
                &request.space_id,
                &request.output_node_id,
                &input.node_id,
                "Presentation generated from selected knowledge",
                now,
            ),
        )
        .await?;
    }
    Ok(())
}

fn validate(request: &CompletePresentationRequest) -> Result<(), String> {
    let signature = request.bytes.get(0..2) == Some(b"PK");
    let primary = request
        .inputs
        .iter()
        .filter(|input| input.role == "primary")
        .count()
        == 1;
    if !signature
        || request.inputs.is_empty()
        || !primary
        || request
            .inputs
            .first()
            .is_none_or(|input| input.role != "primary")
    {
        return Err("Presentation request is invalid".into());
    }
    Ok(())
}
