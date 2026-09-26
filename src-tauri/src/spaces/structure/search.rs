use crate::command_error::CommandError;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tauri::{AppHandle, Runtime};

use crate::spaces::{db, knowledge::search_plan};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct StructureSearchRequest {
    query: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StructureSearchMatch {
    structure_node_id: String,
    node_kind: StructureSearchKind,
    title: String,
    space_id: String,
    space_title: String,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum StructureSearchKind {
    Group,
    Section,
}

#[tauri::command]
pub(crate) async fn search_structure_global<R: Runtime>(
    app: AppHandle<R>,
    request: StructureSearchRequest,
) -> Result<Vec<StructureSearchMatch>, CommandError> {
    let query = request.query.trim();
    if query.is_empty() || query.encode_utf16().count() > 512 {
        return Err("Structure search query must contain 1 to 512 characters".into());
    }
    let plan = search_plan::build(query)?;
    if plan.terms.is_empty() {
        return Ok(Vec::new());
    }
    let pool = db::pool(&app).await?;
    if let Some(fts_query) = plan.fts_query {
        let rows = sqlx::query(
            "SELECT i.id,i.kind,i.title,i.space_id,s.title AS space_title \
             FROM spaces_structure_title_fts f \
             JOIN spaces_structure_nodes i ON i.id=f.structure_node_id AND i.space_id=f.space_id \
             JOIN spaces s ON s.id=i.space_id \
             WHERE i.deleted_at IS NULL AND i.kind IN ('group','section') AND s.archived_at IS NULL \
             AND spaces_structure_title_fts MATCH $1 ORDER BY rank LIMIT 80",
        )
        .bind(fts_query)
        .fetch_all(pool)
        .await
        .map_err(db::sql_error)?;
        if !rows.is_empty() {
            return Ok((rows
                .into_iter()
                .map(decode_match)
                .collect::<Result<_, String>>())?);
        }
    }
    let mut sql = String::from(
        "SELECT i.id,i.kind,i.title,i.space_id,s.title AS space_title \
         FROM spaces_structure_nodes i JOIN spaces s ON s.id=i.space_id \
         WHERE i.deleted_at IS NULL AND i.kind IN ('group','section') AND s.archived_at IS NULL",
    );
    for _ in &plan.terms {
        sql.push_str(" AND instr(lower(i.title),lower(?))>0");
    }
    sql.push_str(" ORDER BY i.title,i.id LIMIT 80");
    let mut search = sqlx::query(&sql);
    for term in plan.terms {
        search = search.bind(term);
    }
    Ok((search
        .fetch_all(pool)
        .await
        .map_err(db::sql_error)?
        .into_iter()
        .map(decode_match)
        .collect::<Result<_, String>>())?)
}

fn decode_match(row: sqlx::sqlite::SqliteRow) -> Result<StructureSearchMatch, String> {
    let node_kind = match row
        .try_get::<String, _>("kind")
        .map_err(db::sql_error)?
        .as_str()
    {
        "group" => StructureSearchKind::Group,
        "section" => StructureSearchKind::Section,
        value => return Err(format!("Invalid structure search kind: {value}")),
    };
    Ok(StructureSearchMatch {
        structure_node_id: row.try_get("id").map_err(db::sql_error)?,
        node_kind,
        title: row.try_get("title").map_err(db::sql_error)?,
        space_id: row.try_get("space_id").map_err(db::sql_error)?,
        space_title: row.try_get("space_title").map_err(db::sql_error)?,
    })
}
