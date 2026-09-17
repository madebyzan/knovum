use serde_json::Value;
use sha2::{Digest, Sha256};

use super::lifecycle::ExportDocumentRequest;
use crate::spaces::derivation::relations;
use crate::spaces::{
    db,
    knowledge::{contracts, indexing::revision},
};

pub async fn apply(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    request: &ExportDocumentRequest,
    data: &mut Value,
    bytes: &[u8],
    rationale: &'static str,
) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    let object = data
        .as_object_mut()
        .ok_or("Document node data is invalid")?;
    object.insert(
        "sourceRevision".into(),
        format!("sha256:{:x}", Sha256::digest(bytes)).into(),
    );
    object.insert("size".into(), (bytes.len() as u64).into());
    object.insert("modifiedAt".into(), now.into());
    object.remove("processingState");
    revision::apply(data)?;
    contracts::validate_node(tx, &request.space_id, &request.output_node_id, "file", data).await?;
    crate::spaces::derivation::persist_output_node(tx, &request.output_node_id, data, now).await?;
    relations::insert(
        tx,
        &relations::derives_from(
            &request.space_id,
            &request.output_node_id,
            &request.source_node_id,
            rationale,
            now,
        ),
    )
    .await?;
    sqlx::query("UPDATE spaces_knowledge_derivations SET status='completed',completed_at=$1,updated_at=$1 WHERE id=$2 AND status='running'")
        .bind(now)
        .bind(&request.id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    Ok(())
}
