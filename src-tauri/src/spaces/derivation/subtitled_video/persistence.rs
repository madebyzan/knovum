use serde_json::Value;
use tauri::AppHandle;

use super::super::video_output::RenderedVideo;
use super::super::{relations, CompletedDerivationOutput};
use crate::spaces::{
    db,
    knowledge::{contracts, indexing::revision},
};

pub struct VideoSubtitleSource {
    pub path: String,
    pub srt: String,
}

pub async fn complete(
    app: &AppHandle,
    id: &str,
    output: RenderedVideo,
) -> Result<CompletedDerivationOutput, String> {
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    let row: (String, String, String, String) = sqlx::query_as(
        "SELECT d.space_id,i.node_id,d.output_node_id,n.data FROM spaces_knowledge_derivations d \
         JOIN spaces_knowledge_derivation_inputs i ON i.derivation_id=d.id AND i.position=0 AND i.role='primary' \
         JOIN spaces_nodes n ON n.id=d.output_node_id WHERE d.id=$1 AND d.status='running'",
    ).bind(id).fetch_optional(&mut *tx).await.map_err(db::sql_error)?
        .ok_or("Subtitle derivation is no longer running")?;
    revalidate(&mut tx, id).await?;
    let now = crate::clock::unix_time_millis()?;
    let mut data: Value = serde_json::from_str(&row.3).map_err(db::json_error)?;
    apply_output(&mut data, &output, now)?;
    contracts::validate_node(&mut tx, &row.0, &row.2, "video", &data).await?;
    let relation = relations::derives_from(
        &row.0,
        &row.2,
        &row.1,
        "Subtitle video created from source video knowledge",
        now,
    );
    let subtitle_id: String = sqlx::query_scalar(
        "SELECT node_id FROM spaces_knowledge_derivation_inputs WHERE derivation_id=$1 AND position=1 AND role='source'",
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    let subtitle_relation = relations::derives_from(
        &row.0,
        &row.2,
        &subtitle_id,
        "Subtitle video rendered from an editable subtitle resource",
        now,
    );
    crate::spaces::derivation::persist_output_node(&mut tx, &row.2, &data, now).await?;
    relations::insert(&mut tx, &relation).await?;
    relations::insert(&mut tx, &subtitle_relation).await?;
    sqlx::query("UPDATE spaces_knowledge_derivations SET status='completed',completed_at=$1,updated_at=$1 WHERE id=$2")
        .bind(now).bind(id).execute(&mut *tx).await.map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(CompletedDerivationOutput {
        node_id: row.2,
        data,
        relation,
    })
}

fn apply_output(data: &mut Value, output: &RenderedVideo, now: i64) -> Result<(), String> {
    let object = data
        .as_object_mut()
        .ok_or("Derived video data is invalid")?;
    object.insert("assetPath".into(), output.path.clone().into());
    object.insert("sourceRevision".into(), output.revision.clone().into());
    object.insert("size".into(), output.size.into());
    object.insert("modifiedAt".into(), now.into());
    object.remove("previewPath");
    object.remove("processingState");
    if let Some(knowledge) = object
        .get_mut("mediaKnowledge")
        .and_then(Value::as_object_mut)
    {
        knowledge.insert("sourceRevision".into(), output.revision.clone().into());
    }
    revision::apply(data).map(|_| ())
}

async fn revalidate(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, id: &str) -> Result<(), String> {
    let valid: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_knowledge_derivations d JOIN spaces_knowledge_derivation_inputs video ON video.derivation_id=d.id AND video.position=0 AND video.role='primary' JOIN spaces_nodes v ON v.id=video.node_id JOIN spaces_knowledge_derivation_inputs subtitles ON subtitles.derivation_id=d.id AND subtitles.position=1 AND subtitles.role='source' JOIN spaces_nodes s ON s.id=subtitles.node_id WHERE d.id=$1 AND v.deleted_at IS NULL AND v.kind='video' AND json_extract(v.data,'$.sourceRevision')=video.source_revision AND s.deleted_at IS NULL AND s.kind='subtitle' AND json_extract(s.data,'$.knowledgeRevision')=subtitles.source_revision")
        .bind(id).fetch_one(&mut **tx).await.map_err(db::sql_error)?;
    if valid == 1 {
        Ok(())
    } else {
        Err("Video or transcript changed during subtitle creation".into())
    }
}
