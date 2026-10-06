use serde::{Deserialize, Serialize};
use sqlx::{Row, Sqlite, Transaction};

use crate::{
    agent_runtime::protocol::{self, control::AssistantExecutionIdentity},
    ai::models::{ModelExecutionTarget, ModelTargetSelection},
    spaces::{assistant::task::identity::TaskIds, db},
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in crate::spaces::assistant) struct Session {
    pub(in crate::spaces::assistant) thread_id: String,
    pub(in crate::spaces::assistant) execution_revision: String,
    pub(in crate::spaces::assistant) control_epoch: u64,
    pub(in crate::spaces::assistant) projection_checkpoint_id: Option<String>,
    pub(in crate::spaces::assistant) input_id: String,
}

impl Session {
    pub(in crate::spaces::assistant) fn identity(
        &self,
        run_id: &str,
    ) -> AssistantExecutionIdentity {
        AssistantExecutionIdentity {
            run_id: run_id.into(),
            thread_id: self.thread_id.clone(),
            execution_revision: self.execution_revision.clone(),
            control_epoch: self.control_epoch,
        }
    }
}

pub(in crate::spaces::assistant) async fn select(
    tx: &mut Transaction<'_, Sqlite>,
    ids: &TaskIds,
    target: &ModelExecutionTarget,
    profile: &crate::ai::settings::contracts::AssistantModelSettings,
    selection: &ModelTargetSelection,
    catalog_digest: &str,
    now: i64,
) -> Result<Session, String> {
    let input_id = ids.input_id();
    let conversation_id = &ids.conversation_id;
    let branch_id = &ids.branch_id;
    let target_json = serde_json::to_string(target).map_err(db::json_error)?;
    let profile_json = serde_json::to_string(profile).map_err(db::json_error)?;
    let selection_json = serde_json::to_string(selection).map_err(db::json_error)?;
    let previous = sqlx::query("SELECT id,execution_revision,frozen_target,model_profile,target_selection,catalog_digest,graph_version,projection_checkpoint_id,
         (SELECT r.state FROM spaces_assistant_tasks t JOIN spaces_runs r ON r.id=t.run_id WHERE t.segment_id=spaces_assistant_execution_segments.id ORDER BY t.created_at DESC,t.rowid DESC LIMIT 1) AS previous_run_state
         FROM spaces_assistant_execution_segments WHERE conversation_id=$1 AND branch_id=$2 ORDER BY created_at DESC,rowid DESC LIMIT 1")
        .bind(conversation_id).bind(branch_id).fetch_optional(&mut **tx).await.map_err(db::sql_error)?;
    if let Some(row) = previous {
        if row
            .try_get::<Option<String>, _>("previous_run_state")
            .map_err(db::sql_error)?
            .as_deref()
            == Some("succeeded")
            && row
                .try_get::<String, _>("frozen_target")
                .map_err(db::sql_error)?
                == target_json
            && row
                .try_get::<String, _>("model_profile")
                .map_err(db::sql_error)?
                == profile_json
            && row
                .try_get::<String, _>("target_selection")
                .map_err(db::sql_error)?
                == selection_json
            && row
                .try_get::<String, _>("catalog_digest")
                .map_err(db::sql_error)?
                == catalog_digest
            && row
                .try_get::<String, _>("graph_version")
                .map_err(db::sql_error)?
                == protocol::GRAPH_VERSION
        {
            return Ok(Session {
                thread_id: row.try_get("id").map_err(db::sql_error)?,
                execution_revision: row.try_get("execution_revision").map_err(db::sql_error)?,
                control_epoch: 0,
                projection_checkpoint_id: row
                    .try_get("projection_checkpoint_id")
                    .map_err(db::sql_error)?,
                input_id,
            });
        }
    }
    let session = Session {
        thread_id: uuid::Uuid::new_v4().to_string(),
        execution_revision: uuid::Uuid::new_v4().to_string(),
        control_epoch: 0,
        projection_checkpoint_id: None,
        input_id,
    };
    sqlx::query("INSERT INTO spaces_assistant_execution_segments(id,conversation_id,branch_id,execution_revision,graph_version,frozen_target,model_profile,target_selection,catalog_digest,created_at)
        VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)").bind(&session.thread_id).bind(conversation_id).bind(branch_id).bind(&session.execution_revision)
        .bind(protocol::GRAPH_VERSION).bind(target_json).bind(profile_json).bind(selection_json).bind(catalog_digest).bind(now)
        .execute(&mut **tx).await.map_err(db::sql_error)?;
    Ok(session)
}
