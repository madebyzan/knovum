use sqlx::{Sqlite, Transaction};

use super::contracts::RestorePackageRequest;
use crate::spaces::db;

pub(super) async fn insert(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
) -> Result<(), String> {
    // Packages contain committed canvas resources and history, never a live Agent workspace.
    for row in &request.external_agent_conversations {
        let error = row.error.clone().unwrap_or_else(|| {
            format!(
                "Imported {} history is not a live runtime session",
                row.state,
            )
        });
        sqlx::query(
            "INSERT INTO spaces_external_agent_conversations(
             id,run_id,space_id,conversation_id,branch_id,runtime_kind,
             runtime_session_id,session_capabilities,title,state,output_publication_state,error,created_at,updated_at)
             VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,'disconnected','committed',$10,$11,$12)",
        )
        .bind(&row.id)
        .bind(&row.run_id)
        .bind(&request.space.id)
        .bind(&row.conversation_id)
        .bind(&row.branch_id)
        .bind(&row.runtime_kind)
        .bind(&row.runtime_session_id)
        .bind(
            row.session_capabilities
                .as_ref()
                .map(sqlx::types::Json),
        )
        .bind(&row.title)
        .bind(error)
        .bind(row.created_at)
        .bind(row.updated_at)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    }
    for row in &request.external_agent_turns {
        let state = if row.state == "running" {
            "failed"
        } else {
            row.state.as_str()
        };
        sqlx::query(
            "INSERT INTO spaces_external_agent_turns(
             id,conversation_id,message_id,state,error,created_at,completed_at)
             VALUES($1,$2,$3,$4,$5,$6,$7)",
        )
        .bind(&row.id)
        .bind(&row.conversation_id)
        .bind(&row.message_id)
        .bind(state)
        .bind(
            row.error
                .as_deref()
                .or((row.state == "running").then_some("Imported active turn cannot resume")),
        )
        .bind(row.created_at)
        .bind(
            row.completed_at
                .or((row.state == "running").then_some(row.created_at)),
        )
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    }
    Ok(())
}
