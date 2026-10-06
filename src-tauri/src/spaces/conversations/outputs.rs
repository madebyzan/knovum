use serde_json::Value;
use sqlx::{Row, Sqlite, Transaction};

use super::ConversationMessageOutput;
use crate::spaces::db;

pub(crate) async fn collect(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    principal: &crate::spaces::authorization::Principal,
) -> Result<Vec<ConversationMessageOutput>, String> {
    let rows = sqlx::query(
        "SELECT n.id,n.kind,n.data FROM spaces_nodes n
         WHERE n.space_id=$1 AND n.deleted_at IS NULL AND (
           EXISTS(SELECT 1 FROM spaces_knowledge_events e
                  WHERE e.space_id=n.space_id AND e.entity_kind='node'
                    AND e.entity_id=n.id AND e.event_kind='created'
                    AND e.actor_kind=$3 AND e.actor_id=$2)
           OR EXISTS(SELECT 1 FROM spaces_action_invocations a
                     JOIN spaces_knowledge_derivations d
                       ON d.id=json_extract(a.outcome,'$.runId')
                     WHERE a.space_id=n.space_id AND a.principal_kind=$4
                       AND a.principal_id=$2 AND a.outcome_kind='accepted_run'
                       AND d.output_node_id=n.id)
         ) ORDER BY n.created_at,n.id",
    )
    .bind(space_id)
    .bind(&principal.id)
    .bind(principal.kind.event_actor_kind())
    .bind(principal.kind.as_str())
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    rows.into_iter()
        .map(|row| {
            let encoded: String = row.try_get("data").map_err(db::sql_error)?;
            let data: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
            Ok(ConversationMessageOutput {
                node_id: row.try_get("id").map_err(db::sql_error)?,
                kind: row.try_get("kind").map_err(db::sql_error)?,
                title: data["title"]
                    .as_str()
                    .ok_or("Conversation output has no node title")?
                    .to_owned(),
            })
        })
        .collect()
}
