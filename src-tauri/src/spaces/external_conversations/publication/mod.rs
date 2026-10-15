use std::path::Path;

use serde_json::Value;
use sqlx::{Sqlite, Transaction};
use tauri::AppHandle;

use crate::{
    foundation::app_error::AppError,
    spaces::{
        conversations::ConversationMessageOutput,
        db, knowledge,
        nodes::{self, contracts::InsertNode},
        structure,
    },
};

pub(super) async fn publish(
    app: &AppHandle,
    conversation_id: &str,
    run_id: &str,
    space_id: &str,
    workspace: &Path,
) -> Result<(), AppError> {
    let pool = db::pool(app).await?;
    let _ownership = super::publication_ownership::acquire(run_id).await?;
    let (publication, cancelled): (String, Option<i64>) = sqlx::query_as(
        "SELECT c.output_publication_state,r.cancel_requested_at FROM spaces_external_agent_conversations c
         JOIN spaces_runs r ON r.id=c.run_id WHERE c.id=$1 AND c.run_id=$2",
    ).bind(conversation_id).bind(run_id).fetch_one(pool).await.map_err(db::sql_error)?;
    if publication == "committed" {
        discard_committed_workspace(workspace);
        return Ok(());
    }
    if cancelled.is_some() {
        return Err("Cancelled Agent outputs cannot be published".into());
    }
    crate::spaces::runs::wait_for_children(pool, run_id).await?;
    let claim = sqlx::query("UPDATE spaces_external_agent_conversations SET output_publication_state='publishing' WHERE id=$1 AND output_publication_state IN ('pending','failed')")
        .bind(conversation_id).execute(pool).await.map_err(db::sql_error)?;
    db::require_one(claim, "Agent output publication is already active")?;
    super::staging::cleanup(app, pool, run_id).await?;
    let outputs = super::staging::stage(app, pool, run_id, workspace).await?;
    let publication = commit(
        pool,
        conversation_id,
        run_id,
        space_id,
        &outputs,
        crate::spaces::assets::enrichment::jobs::intent::selection(app)?.as_ref(),
    )
    .await;
    if let Err(error) = publication {
        return match super::staging::cleanup(app, pool, run_id).await {
            Ok(()) => Err(error),
            Err(cleanup) => {
                log::error!(
                    "Agent output publication failed: {error:?}; cleanup failed: {cleanup}"
                );
                Err(AppError::new(
                    "artifact_cleanup_failed",
                    format!("{}; staged output cleanup failed: {cleanup}", error.message),
                ))
            }
        };
    }
    discard_committed_workspace(workspace);
    Ok(())
}

fn discard_committed_workspace(workspace: &Path) {
    if let Err(error) = crate::external_agents::artifacts::discard(workspace) {
        log::warn!("Could not remove committed Agent workspace: {error}");
    }
}

pub(super) fn dimensions(kind: &str) -> (f64, f64) {
    match kind {
        "image" => (256.0, 156.0),
        "video" => (304.0, 170.0),
        "audio" => (288.0, 80.0),
        "markdown" => (260.0, 184.0),
        "code" => (304.0, 224.0),
        _ => (256.0, 192.0),
    }
}

async fn commit(
    pool: &sqlx::SqlitePool,
    conversation_id: &str,
    run_id: &str,
    space_id: &str,
    outputs: &[InsertNode],
    enrichment_target: Option<&crate::ai::tasks::TaskTargetSelection>,
) -> Result<(), AppError> {
    let mut tx = db::begin_write(pool).await?;
    let now = crate::foundation::clock::unix_time_millis()?;
    let (snapshot, is_retry): (String, bool) = sqlx::query_as(
        "SELECT r.input_snapshot,c.output_publication_error IS NOT NULL FROM spaces_runs r JOIN spaces_external_agent_conversations c ON c.run_id=r.id
         WHERE r.id=$1 AND r.cancel_requested_at IS NULL AND r.state<>'cancelled'
         AND c.output_publication_state<>'committed' ",
    )
    .bind(run_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    let sources = current_sources(&mut tx, space_id, &snapshot).await?;
    let principal = crate::spaces::authorization::Principal {
        kind: crate::spaces::authorization::PrincipalKind::ExternalAgent,
        id: conversation_id.to_owned(),
    };
    let mut references =
        crate::spaces::conversations::outputs::collect(&mut tx, space_id, &principal).await?;
    if !outputs.is_empty() {
        insert_outputs(
            &mut tx,
            space_id,
            conversation_id,
            outputs,
            &sources,
            now,
            enrichment_target,
        )
        .await?;
        for output in outputs {
            references.push(ConversationMessageOutput {
                node_id: output.id.clone(),
                title: output.data["title"]
                    .as_str()
                    .ok_or("Agent output has no node title")?
                    .to_owned(),
                kind: output.kind.clone(),
            });
        }
    }
    if !references.is_empty() {
        attach_outputs(&mut tx, conversation_id, &references).await?;
    }
    let settlement = crate::spaces::runs::child_settlement(&mut tx, run_id)
        .await?
        .ok_or("Agent has unfinished child Actions")?;
    let state = if settlement == crate::spaces::runs::RunState::Succeeded {
        "succeeded"
    } else {
        "completed_with_errors"
    };
    sqlx::query(
        "UPDATE spaces_external_agent_conversations SET state='closed',output_publication_state='committed',
         output_publication_error=NULL,error=$3,updated_at=$1 WHERE id=$2",
    )
    .bind(now)
    .bind(conversation_id)
    .bind((settlement != crate::spaces::runs::RunState::Succeeded).then_some("Some creation tasks failed. Completed resources remain available."))
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    let updated = sqlx::query("UPDATE spaces_runs SET state=$3,state_version=state_version+1,updated_at=$1,completed_at=$1 WHERE id=$2 AND (completed_at IS NULL OR (state='failed' AND $4)) AND cancel_requested_at IS NULL")
        .bind(now)
        .bind(run_id)
        .bind(state)
        .bind(is_retry)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    db::require_one(updated, "Agent Run changed before publication")?;
    super::staging::release(&mut tx, run_id).await?;
    tx.commit().await.map_err(db::sql_error)?;
    crate::spaces::runs::announce_change();
    Ok(())
}

async fn current_sources(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    snapshot: &str,
) -> Result<Vec<String>, AppError> {
    let input: Value = serde_json::from_str(snapshot).map_err(db::json_error)?;
    let attachments = input
        .get("attachments")
        .and_then(Value::as_array)
        .ok_or("Agent run has no attachment manifest")?;
    let mut sources = Vec::new();
    for entry in attachments {
        if entry["source"] != "node" {
            continue;
        }
        let node_id = entry["nodeId"]
            .as_str()
            .ok_or("Agent attachment has no canvas node identity")?;
        let present: Option<i64> = sqlx::query_scalar(
            "SELECT 1 FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND deleted_at IS NULL",
        )
        .bind(node_id)
        .bind(space_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(db::sql_error)?;
        if present.is_some() && !sources.iter().any(|source| source == node_id) {
            sources.push(node_id.to_owned());
        }
    }
    Ok(sources)
}

async fn insert_outputs(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    principal_id: &str,
    outputs: &[InsertNode],
    sources: &[String],
    now: i64,
    enrichment_target: Option<&crate::ai::tasks::TaskTargetSelection>,
) -> Result<(), AppError> {
    let (mut snapshot, revision) = structure::repository::load_in_transaction(tx, space_id).await?;
    let before = structure::history::capture(tx, space_id, &snapshot).await?;
    let mut anchor = sources.first().cloned().or_else(|| {
        snapshot.structure.iter().rev().find_map(|item| match item {
            structure::StructureNode::Content {
                node_id,
                parent_id: None,
                ..
            } => Some(node_id.clone()),
            _ => None,
        })
    });
    for output in outputs {
        let mut node = output.clone();
        let requested = structure::Size {
            width: node.width,
            height: node.height,
        };
        let size = nodes::fitted_content_size(&node.data, &requested, node.sizing_mode)
            .unwrap_or(requested);
        let (x, y) = match &anchor {
            Some(id) => structure::place_new_content(
                &snapshot.structure,
                &snapshot.layouts,
                std::slice::from_ref(id),
                size.width,
                size.height,
            )?,
            None => structure::place_new_content(
                &snapshot.structure,
                &snapshot.layouts,
                &[],
                size.width,
                size.height,
            )?,
        };
        node.initial_x = x;
        node.initial_y = y;
        crate::spaces::knowledge::validation::normalize_node(
            tx,
            space_id,
            &node.kind,
            &mut node.data,
        )
        .await?;
        structure::creation::append_content(&mut snapshot, std::slice::from_ref(&node), None)?;
        nodes::insert_rows_with_actor(
            tx,
            space_id,
            std::slice::from_ref(&node),
            knowledge::events::EventActor::new("external_agent", Some(principal_id)),
            now,
            enrichment_target,
        )
        .await?;
        anchor = Some(node.id);
    }
    let next =
        structure::repository::save_in_transaction(tx, space_id, revision, &snapshot).await?;
    let after = structure::history::capture(tx, space_id, &snapshot).await?;
    structure::history::record(tx, space_id, &before, &after, next).await?;
    Ok(())
}

async fn attach_outputs(
    tx: &mut Transaction<'_, Sqlite>,
    conversation_id: &str,
    outputs: &[ConversationMessageOutput],
) -> Result<(), AppError> {
    let publication = sqlx::query("UPDATE spaces_conversation_messages SET content=json_set(content,'$.outputs',json($1)) WHERE id=(SELECT message_id FROM spaces_external_agent_turns WHERE conversation_id=$2 AND state='completed' ORDER BY created_at DESC LIMIT 1)")
        .bind(sqlx::types::Json(outputs))
        .bind(conversation_id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    db::require_one(
        publication,
        "Agent output has no completed conversation turn",
    )?;
    Ok(())
}

#[cfg(test)]
mod tests;
