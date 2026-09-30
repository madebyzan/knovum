use serde::Serialize;
use sqlx::{Row, SqliteConnection};

use crate::spaces::db;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ContentsPage {
    pub space_id: String,
    pub title: String,
    pub total_nodes: i64,
    pub nodes: Vec<ContentNode>,
    pub next_offset: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ContentNode {
    pub node_id: String,
    pub kind: String,
    pub title: String,
    pub knowledge_revision: Option<String>,
    pub source_revision: Option<String>,
    pub indexed: bool,
    pub updated_at: i64,
}

pub(crate) async fn list_contents(
    connection: &mut SqliteConnection,
    space_id: &str,
    offset: i64,
    limit: i64,
) -> Result<ContentsPage, String> {
    if offset < 0 || !(1..=100).contains(&limit) {
        return Err(
            "space-contents-invalid: offset must be nonnegative and limit must be 1–100".into(),
        );
    }
    let title: String =
        sqlx::query_scalar("SELECT title FROM spaces WHERE id=$1 AND archived_at IS NULL")
            .bind(space_id)
            .fetch_optional(&mut *connection)
            .await
            .map_err(db::sql_error)?
            .ok_or("space-unavailable: current Space does not exist or is archived")?;
    let total_nodes: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_nodes WHERE space_id=$1 AND deleted_at IS NULL \
         AND kind NOT IN ('credential','portal')",
    )
    .bind(space_id)
    .fetch_one(&mut *connection)
    .await
    .map_err(db::sql_error)?;
    let rows = sqlx::query(
        "SELECT n.id,n.kind,json_extract(n.data,'$.title') title, \
         json_extract(n.data,'$.knowledgeRevision') knowledge_revision, \
         json_extract(n.data,'$.sourceRevision') source_revision, \
         EXISTS(SELECT 1 FROM spaces_current_knowledge_chunks c \
                WHERE c.space_id=n.space_id AND c.node_id=n.id) is_indexed,n.updated_at \
         FROM spaces_nodes n WHERE n.space_id=$1 AND n.deleted_at IS NULL \
         AND n.kind NOT IN ('credential','portal') ORDER BY n.id LIMIT $2 OFFSET $3",
    )
    .bind(space_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&mut *connection)
    .await
    .map_err(db::sql_error)?;
    let nodes = rows
        .into_iter()
        .map(|row| {
            Ok(ContentNode {
                node_id: row.try_get("id").map_err(db::sql_error)?,
                kind: row.try_get("kind").map_err(db::sql_error)?,
                title: row.try_get("title").map_err(db::sql_error)?,
                knowledge_revision: row.try_get("knowledge_revision").map_err(db::sql_error)?,
                source_revision: row.try_get("source_revision").map_err(db::sql_error)?,
                indexed: row.try_get("is_indexed").map_err(db::sql_error)?,
                updated_at: row.try_get("updated_at").map_err(db::sql_error)?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let end = offset
        .checked_add(nodes.len() as i64)
        .ok_or("space-contents-invalid: offset overflow")?;
    Ok(ContentsPage {
        space_id: space_id.to_owned(),
        title,
        total_nodes,
        nodes,
        next_offset: (end < total_nodes).then_some(end),
    })
}
