use serde_json::Value;
use sqlx::{Row, Sqlite, Transaction};

use crate::spaces::db;

pub(super) async fn rows(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<Vec<Value>, String> {
    reject_active(tx, space_id).await?;
    let rows = sqlx::query(
        "SELECT id,space_id AS spaceId,output_node_id AS outputNodeId,\
         action_kind AS actionKind,action_version AS actionVersion,engine,status,error,\
         created_at AS createdAt,updated_at AS updatedAt,completed_at AS completedAt \
         FROM spaces_knowledge_derivations WHERE space_id=? AND EXISTS(SELECT 1 FROM spaces_nodes n WHERE n.id=output_node_id AND n.space_id=spaces_knowledge_derivations.space_id AND n.deleted_at IS NULL) ORDER BY created_at,id",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let mut values = crate::sqlite::rows_to_json(rows)?;
    for value in &mut values {
        let id = value
            .get("id")
            .and_then(Value::as_str)
            .ok_or("Exported derivation ID is unavailable")?
            .to_owned();
        let rows = sqlx::query(
            "SELECT CASE WHEN n.deleted_at IS NULL AND n.space_id=d.space_id THEN i.node_id ELSE NULL END AS nodeId,i.role,i.source_revision AS sourceRevision,i.source_identity AS sourceIdentity,i.source_snapshot AS sourceSnapshot \
             FROM spaces_knowledge_derivation_inputs i JOIN spaces_knowledge_derivations d ON d.id=i.derivation_id LEFT JOIN spaces_nodes n ON n.id=i.node_id WHERE i.derivation_id=? ORDER BY i.position",
        )
        .bind(&id)
        .fetch_all(&mut **tx)
        .await
        .map_err(db::sql_error)?;
        let inputs = rows
            .into_iter()
            .map(|row| {
                Ok(serde_json::json!({
                    "nodeId": row.try_get::<Option<String>, _>("nodeId").map_err(db::sql_error)?,
                    "sourceIdentity": row.try_get::<String, _>("sourceIdentity").map_err(db::sql_error)?,
                    "sourceSnapshot": super::parse_json(&row, "sourceSnapshot")?,
                    "role": row.try_get::<String, _>("role").map_err(db::sql_error)?,
                    "sourceRevision": row.try_get::<String, _>("sourceRevision").map_err(db::sql_error)?,
                }))
            })
            .collect::<Result<Vec<_>, String>>()?;
        value
            .as_object_mut()
            .ok_or("Exported derivation must be an object")?
            .insert("inputs".into(), Value::Array(inputs));
        let grounded = sqlx::query(
            "SELECT citations,provider,model FROM spaces_grounded_documents WHERE derivation_id=?",
        )
        .bind(&id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(db::sql_error)?;
        if let Some(row) = grounded {
            let citations: String = row.try_get("citations").map_err(db::sql_error)?;
            value.as_object_mut().ok_or("Exported derivation must be an object")?.insert(
                "groundedDocument".into(),
                serde_json::json!({
                    "citations": serde_json::from_str::<Value>(&citations).map_err(db::json_error)?,
                    "provider": row.try_get::<String, _>("provider").map_err(db::sql_error)?,
                    "model": row.try_get::<String, _>("model").map_err(db::sql_error)?,
                }),
            );
        }
        let image = sqlx::query("SELECT intent,style,aspect_ratio AS aspectRatio,provider,model,revised_prompt AS revisedPrompt FROM spaces_generated_images WHERE derivation_id=?")
            .bind(&id).fetch_optional(&mut **tx).await.map_err(db::sql_error)?;
        if let Some(row) = image {
            value.as_object_mut().ok_or("Exported derivation must be an object")?.insert(
                "generatedImage".into(), serde_json::json!({
                    "intent": row.try_get::<String, _>("intent").map_err(db::sql_error)?,
                    "style": row.try_get::<String, _>("style").map_err(db::sql_error)?,
                    "aspectRatio": row.try_get::<String, _>("aspectRatio").map_err(db::sql_error)?,
                    "provider": row.try_get::<String, _>("provider").map_err(db::sql_error)?,
                    "model": row.try_get::<String, _>("model").map_err(db::sql_error)?,
                    "revisedPrompt": row.try_get::<Option<String>, _>("revisedPrompt").map_err(db::sql_error)?,
                }),
            );
        }
        let video = sqlx::query("SELECT prompt,aspect_ratio AS aspectRatio,duration_seconds AS durationSeconds,provider,model,operation_id AS operationId FROM spaces_generated_videos WHERE derivation_id=?")
            .bind(&id).fetch_optional(&mut **tx).await.map_err(db::sql_error)?;
        if let Some(row) = video {
            value.as_object_mut().ok_or("Exported derivation must be an object")?.insert(
                "generatedVideo".into(), serde_json::json!({
                    "prompt": row.try_get::<String, _>("prompt").map_err(db::sql_error)?,
                    "aspectRatio": row.try_get::<String, _>("aspectRatio").map_err(db::sql_error)?,
                    "durationSeconds": row.try_get::<i64, _>("durationSeconds").map_err(db::sql_error)?,
                    "provider": row.try_get::<String, _>("provider").map_err(db::sql_error)?,
                    "model": row.try_get::<String, _>("model").map_err(db::sql_error)?,
                    "operationId": row.try_get::<String, _>("operationId").map_err(db::sql_error)?,
                }),
            );
        }
    }
    Ok(values)
}

async fn reject_active(tx: &mut Transaction<'_, Sqlite>, space_id: &str) -> Result<(), String> {
    let active: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_knowledge_derivations \
         WHERE space_id=$1 AND status IN ('queued','running')",
    )
    .bind(space_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    if active > 0 {
        return Err("Wait for active knowledge derivations to finish before exporting".into());
    }
    Ok(())
}
