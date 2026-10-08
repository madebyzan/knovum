use serde_json::Value;
use tauri::AppHandle;

use super::super::{
    inputs, pending_output, relations, video_output::RenderedVideo, CompletedDerivationOutput,
};
use crate::{
    app_error::AppError,
    spaces::{db, knowledge::model::revision},
};

pub async fn complete(
    app: &AppHandle,
    id: &str,
    output: RenderedVideo,
) -> Result<CompletedDerivationOutput, AppError> {
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    let row: (String, String, String, String, String) = sqlx::query_as(
        "SELECT d.space_id,i.node_id,d.output_node_id,n.data,json_extract(source.data,'$.assetPath') FROM spaces_knowledge_derivations d \
         JOIN spaces_knowledge_derivation_inputs i ON i.derivation_id=d.id AND i.position=0 AND i.role='primary' \
         JOIN spaces_nodes source ON source.id=i.node_id AND source.space_id=d.space_id \
         JOIN spaces_nodes n ON n.id=d.output_node_id WHERE d.id=$1 AND d.status='running'",
    ).bind(id).fetch_optional(&mut *tx).await.map_err(db::sql_error)?.ok_or("Video edit is no longer rendering")?;
    inputs::validate_current(&mut tx, id).await?;
    let now = crate::clock::unix_time_millis()?;
    let mut data: Value = serde_json::from_str(&row.3).map_err(db::json_error)?;
    pending_output::validate_separate_asset(&data, &row.4)?;
    apply_output(&mut data, &output, now)?;
    crate::spaces::knowledge::validation::validate_node(&mut tx, &row.0, "video", &data).await?;
    let relation = relations::derives_from(
        &row.0,
        &row.2,
        &row.1,
        "Video rendered from a non-destructive edit project",
        now,
    );
    crate::spaces::derivation::persist_output_node(&mut tx, &row.2, &data, now).await?;
    relations::insert(&mut tx, &relation).await?;
    sqlx::query("UPDATE spaces_knowledge_derivations SET status='completed',completed_at=$1,updated_at=$1 WHERE id=$2")
        .bind(now).bind(id).execute(&mut *tx).await.map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(CompletedDerivationOutput {
        node_id: row.2,
        data,
        relation,
    })
}

fn apply_output(data: &mut Value, output: &RenderedVideo, now: i64) -> Result<(), AppError> {
    let object = data
        .as_object_mut()
        .ok_or("Rendered video data is invalid")?;
    object.insert("assetPath".into(), output.path.clone().into());
    object.insert("sourceRevision".into(), output.revision.clone().into());
    object.insert("size".into(), output.size.into());
    object.insert("modifiedAt".into(), now.into());
    object.remove("previewPath");
    object.remove("processingState");
    object.remove("videoEditProject");
    object.remove("mediaKnowledge");
    object.remove("mediaAnalysis");
    revision::apply(data).map(|_| ()).map_err(AppError::from)
}
