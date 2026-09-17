use serde_json::Value;
use sqlx::Row;
use tauri::{AppHandle, Runtime};

use crate::spaces::db;

#[tauri::command]
pub async fn list_knowledge_health_issues<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
) -> Result<Vec<Value>, String> {
    let pool = db::pool(&app).await?;
    let mut issues = Vec::new();

    append_node_issues(
        &mut issues,
        sqlx::query(
            "SELECT n.id,n.data FROM spaces_nodes n \
             WHERE n.space_id=$1 AND n.kind='object' AND n.object_type_id='system:claim' AND n.deleted_at IS NULL \
               AND json_extract(n.data,'$.status') IN ('draft','accepted','contested') \
               AND NOT EXISTS(SELECT 1 FROM spaces_relations r WHERE r.target_id=n.id AND r.kind='supports' AND r.status<>'invalidated')",
        ).bind(&space_id).fetch_all(pool).await.map_err(db::sql_error)?,
        "unsupported-claim",
        "warning",
    )?;
    append_node_issues(
        &mut issues,
        sqlx::query(
            "SELECT DISTINCT n.id,n.data FROM spaces_nodes n JOIN spaces_relations r ON r.target_id=n.id \
             WHERE n.space_id=$1 AND n.kind='object' AND n.object_type_id='system:claim' AND n.deleted_at IS NULL \
               AND json_extract(n.data,'$.status')='accepted' AND r.kind='contradicts' AND r.status<>'invalidated'",
        ).bind(&space_id).fetch_all(pool).await.map_err(db::sql_error)?,
        "contradicted-claim",
        "critical",
    )?;
    append_node_issues(
        &mut issues,
        sqlx::query(
            "SELECT n.id,n.data FROM spaces_nodes n \
             WHERE n.space_id=$1 AND n.kind='object' AND n.object_type_id='system:evidence' AND n.deleted_at IS NULL \
               AND NOT EXISTS(SELECT 1 FROM spaces_provenance p WHERE p.target_kind='node' AND p.target_id=n.id AND p.status='active')",
        ).bind(&space_id).fetch_all(pool).await.map_err(db::sql_error)?,
        "ungrounded-evidence",
        "warning",
    )?;
    append_node_issues(
        &mut issues,
        sqlx::query(
            "SELECT DISTINCT n.id,n.data FROM spaces_nodes n JOIN spaces_relations r ON r.target_id=n.id \
             WHERE n.space_id=$1 AND n.kind='object' AND n.object_type_id='system:question' AND n.deleted_at IS NULL \
               AND json_extract(n.data,'$.status')='open' AND r.kind='answers' AND r.status<>'invalidated'",
        ).bind(&space_id).fetch_all(pool).await.map_err(db::sql_error)?,
        "answered-open-question",
        "info",
    )?;

    let stale = sqlx::query(
        "SELECT p.id,p.target_id,p.source_node_id,p.source_revision,s.data \
         FROM spaces_provenance p JOIN spaces_nodes s ON s.id=p.source_node_id AND s.deleted_at IS NULL \
         WHERE p.space_id=$1 AND p.target_kind='node' AND p.status='active' \
           AND COALESCE(json_extract(s.data,'$.sourceRevision'),json_extract(s.data,'$.knowledgeRevision'),'')<>'' \
           AND p.source_revision<>COALESCE(json_extract(s.data,'$.sourceRevision'),json_extract(s.data,'$.knowledgeRevision'),'')",
    ).bind(&space_id).fetch_all(pool).await.map_err(db::sql_error)?;
    for row in stale {
        let data = parse_data(&row)?;
        issues.push(serde_json::json!({
            "id": format!("stale-provenance:{}", row.try_get::<String,_>("id").map_err(db::sql_error)?),
            "kind": "stale-provenance",
            "severity": "warning",
            "nodeId": row.try_get::<String,_>("target_id").map_err(db::sql_error)?,
            "sourceNodeId": row.try_get::<String,_>("source_node_id").map_err(db::sql_error)?,
            "title": data.get("title").and_then(Value::as_str).unwrap_or("Source changed"),
            "sourceRevision": row.try_get::<String,_>("source_revision").map_err(db::sql_error)?,
            "currentRevision": current_revision(&data),
        }));
    }
    Ok(issues)
}

fn append_node_issues(
    target: &mut Vec<Value>,
    rows: Vec<sqlx::sqlite::SqliteRow>,
    kind: &str,
    severity: &str,
) -> Result<(), String> {
    for row in rows {
        let id: String = row.try_get("id").map_err(db::sql_error)?;
        let data = parse_data(&row)?;
        target.push(serde_json::json!({
            "id": format!("{kind}:{id}"),
            "kind": kind,
            "severity": severity,
            "nodeId": id,
            "title": data.get("title").and_then(Value::as_str).unwrap_or("Knowledge Object"),
        }));
    }
    Ok(())
}

fn parse_data(row: &sqlx::sqlite::SqliteRow) -> Result<Value, String> {
    let raw: String = row.try_get("data").map_err(db::sql_error)?;
    serde_json::from_str(&raw).map_err(db::json_error)
}

fn current_revision(data: &Value) -> &str {
    data.get("sourceRevision")
        .and_then(Value::as_str)
        .or_else(|| data.get("knowledgeRevision").and_then(Value::as_str))
        .unwrap_or_default()
}
