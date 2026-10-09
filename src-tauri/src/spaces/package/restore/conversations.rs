use sqlx::{Sqlite, Transaction};

use super::contracts::RestorePackageRequest;
use crate::spaces::db;

pub(super) async fn insert(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
) -> Result<(), String> {
    for row in &request.conversations {
        sqlx::query(
            "INSERT INTO spaces_conversations(id,space_id,title,active_branch_id,archived_at,created_at,updated_at)
             VALUES($1,$2,$3,$4,$5,$6,$7)",
        ).bind(&row.id).bind(&request.space.id).bind(&row.title)
            .bind(&row.active_branch_id).bind(row.archived_at).bind(row.created_at).bind(row.updated_at)
            .execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    for row in &request.conversation_messages {
        sqlx::query(
            "INSERT INTO spaces_conversation_messages(id,conversation_id,branch_id,parent_message_id,role,content,created_at)
             VALUES($1,$2,$3,NULL,$4,$5,$6)",
        ).bind(&row.id).bind(&row.conversation_id).bind(&row.branch_id)
            .bind(&row.role)
            .bind(sqlx::types::Json(&row.content))
            .bind(row.created_at).execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    for row in &request.conversation_messages {
        if let Some(parent_id) = &row.parent_message_id {
            sqlx::query("UPDATE spaces_conversation_messages SET parent_message_id=$1 WHERE id=$2")
                .bind(parent_id)
                .bind(&row.id)
                .execute(&mut **tx)
                .await
                .map_err(db::sql_error)?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/conversations.rs"]
mod tests;
