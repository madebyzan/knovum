mod sources;

use std::path::Path;

use sqlx::{Sqlite, Transaction};
use tauri::AppHandle;

use self::sources::ResolvedSources;
use crate::{
    foundation::app_error::AppError,
    spaces::{
        conversations::ConversationMessageOutput,
        db,
        derivation::placement::{self, DerivedOutputs, SetMember},
        knowledge,
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
        drain_committed_workspace(workspace);
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
    let publication = commit(pool, conversation_id, run_id, space_id, &outputs).await;
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
    drain_committed_workspace(workspace);
    Ok(())
}

/// What was published is now the Space's; the workspace is emptied for the conversation's next
/// turn, which publishes only what it writes itself.
fn drain_committed_workspace(workspace: &Path) {
    if let Err(error) = crate::external_agents::artifacts::clear(workspace) {
        log::warn!("Could not empty the committed Agent workspace: {error}");
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
    outputs: &[super::staging::StagedOutput],
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
    let sources = sources::resolve(
        &mut tx,
        space_id,
        conversation_id,
        run_id,
        &snapshot,
        outputs,
    )
    .await?;
    let principal = crate::spaces::authorization::Principal {
        kind: crate::spaces::authorization::PrincipalKind::ExternalAgent,
        id: conversation_id.to_owned(),
    };
    let mut references =
        crate::spaces::conversations::outputs::collect(&mut tx, space_id, &principal).await?;
    if !outputs.is_empty() {
        insert_outputs(&mut tx, space_id, conversation_id, outputs, &sources, now).await?;
        for output in outputs {
            references.push(ConversationMessageOutput {
                node_id: output.node.id.clone(),
                title: output.node.data["title"]
                    .as_str()
                    .ok_or("Agent output has no node title")?
                    .to_owned(),
                kind: output.node.kind.clone(),
                workspace_path: Some(output.workspace_path.clone()),
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

async fn insert_outputs(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    principal_id: &str,
    outputs: &[super::staging::StagedOutput],
    sources: &ResolvedSources,
    now: i64,
) -> Result<(), AppError> {
    let (mut snapshot, revision) = structure::repository::load_in_transaction(tx, space_id).await?;
    let before = structure::history::capture(tx, space_id, &snapshot).await?;
    let cells = placement::plan(
        tx,
        space_id,
        &snapshot.structure,
        &snapshot.layouts,
        DerivedOutputs {
            sources: &sources.placement_sources,
            reserved: &[],
            members: outputs
                .iter()
                .map(|output| set_member(&output.node))
                .collect(),
        },
    )
    .await?;
    for ((output, cell), output_sources) in
        outputs.iter().zip(&cells).zip(sources.per_output.iter())
    {
        let mut node = output.node.clone();
        node.initial_x = cell.x;
        node.initial_y = cell.y;
        node.width = cell.size.width;
        node.height = cell.size.height;
        node.sizing_mode = cell.sizing_mode;
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
        )
        .await?;
        for source in output_sources {
            crate::spaces::derivation::relations::derives_from(
                tx,
                space_id,
                &node.id,
                &source.node_id,
                source.lineage,
                now,
            )
            .await?;
        }
    }
    structure::repository::save_in_transaction(tx, space_id, revision, &mut snapshot).await?;
    let after = structure::history::capture(tx, space_id, &snapshot).await?;
    structure::history::record(tx, space_id, &before, &after).await?;
    Ok(())
}

/// One staged output as a member of the set its turn publishes: the footprint its content reports,
/// or the size its kind was staged with.
fn set_member(node: &InsertNode) -> SetMember {
    let requested = structure::Size {
        width: node.width,
        height: node.height,
    };
    SetMember::new(&node.data, &requested, node.sizing_mode)
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
