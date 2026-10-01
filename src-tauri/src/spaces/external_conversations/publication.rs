use std::path::Path;

use serde_json::Value;
use sqlx::{Sqlite, Transaction};
use tauri::AppHandle;

use crate::spaces::nodes::contracts::InsertNode;
use crate::spaces::{assistant::AssistantMessageOutput, db, knowledge, nodes, structure};

pub(super) struct PreparedOutput {
    pub(super) node: InsertNode,
}

pub(super) async fn publish(
    app: &AppHandle,
    conversation_id: &str,
    run_id: &str,
    space_id: &str,
    workspace: &Path,
) -> Result<(), String> {
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
    let result = commit(
        pool,
        conversation_id,
        run_id,
        space_id,
        &outputs,
        crate::spaces::assets::enrichment::jobs::intent::selection(app)?.as_ref(),
    )
    .await;
    if let Err(error) = result {
        return match super::staging::cleanup(app, pool, run_id).await {
            Ok(()) => Err(error),
            Err(cleanup) => Err(format!("{error}; staged output cleanup failed: {cleanup}")),
        };
    }
    discard_committed_workspace(workspace);
    Ok(())
}

fn discard_committed_workspace(workspace: &Path) {
    if let Err(error) = crate::ai::capabilities::agent_artifacts::discard(workspace) {
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
    outputs: &[PreparedOutput],
    enrichment_target: Option<&crate::ai::tasks::TaskTargetSelection>,
) -> Result<(), String> {
    let mut tx = db::begin_write(pool).await?;
    let now = crate::clock::unix_time_millis()?;
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
        crate::spaces::assistant::outputs::collect(&mut tx, space_id, &principal).await?;
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
            references.push(AssistantMessageOutput {
                node_id: output.node.id.clone(),
                title: output.node.data["title"]
                    .as_str()
                    .ok_or("Agent output has no node title")?
                    .to_owned(),
                kind: output.node.kind.clone(),
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
) -> Result<Vec<String>, String> {
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
    outputs: &[PreparedOutput],
    sources: &[String],
    now: i64,
    enrichment_target: Option<&crate::ai::tasks::TaskTargetSelection>,
) -> Result<(), String> {
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
        let mut node = output.node.clone();
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
        knowledge::contracts::normalize_node(tx, space_id, &node.kind, &mut node.data).await?;
        structure::creation::append_content(&mut snapshot, std::slice::from_ref(&node), None)?;
        nodes::insert_rows_with_actor(
            tx,
            space_id,
            std::slice::from_ref(&node),
            knowledge::events::EventActor::new("agent", Some(principal_id)),
            now,
            enrichment_target,
        )
        .await?;
        anchor = Some(node.id);
    }
    let next =
        structure::repository::save_in_transaction(tx, space_id, revision, &snapshot).await?;
    let after = structure::history::capture(tx, space_id, &snapshot).await?;
    structure::history::record(tx, space_id, &before, &after, None, None, next).await?;
    Ok(())
}

async fn attach_outputs(
    tx: &mut Transaction<'_, Sqlite>,
    conversation_id: &str,
    outputs: &[AssistantMessageOutput],
) -> Result<(), String> {
    let encoded = serde_json::to_string(outputs).map_err(db::json_error)?;
    let result = sqlx::query("UPDATE spaces_assistant_messages SET content=json_set(content,'$.outputs',json($1)) WHERE id=(SELECT assistant_message_id FROM spaces_external_agent_turns WHERE conversation_id=$2 AND state='completed' ORDER BY created_at DESC LIMIT 1)")
        .bind(encoded)
        .bind(conversation_id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    db::require_one(result, "Agent output has no completed conversation turn")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    async fn fixture() -> sqlx::SqlitePool {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        db::migrate(&pool).await.unwrap();
        sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
            .execute(&pool).await.unwrap();
        let mut tx = db::begin_write(&pool).await.unwrap();
        structure::initialize_empty_in_transaction(&mut tx, "space")
            .await
            .unwrap();
        tx.commit().await.unwrap();
        sqlx::query("INSERT INTO spaces_assistant_conversations(id,space_id,title,mode,created_at,updated_at,active_branch_id) VALUES('conversation','space','Agent','ask',1,1,'conversation')")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO spaces_runs(id,home_space_id,principal_kind,principal_id,kind,state,policy_snapshot,input_snapshot,budget,created_at,updated_at) VALUES('run','space','external_agent','session','external_agent','running','{}','{\"attachments\":[]}','{}',1,1)")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO spaces_external_agent_conversations(id,run_id,space_id,runtime_kind,title,state,created_at,updated_at,assistant_conversation_id,branch_id) VALUES('session','run','space','codex','Agent','connected',1,1,'conversation','conversation')")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO spaces_assistant_messages(id,conversation_id,branch_id,role,content,created_at) VALUES('reply','conversation','conversation','assistant','{\"parts\":[{\"type\":\"text\",\"text\":\"\"}],\"runId\":\"run\"}',1)")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO spaces_external_agent_turns(id,conversation_id,state,created_at,completed_at,assistant_message_id) VALUES('turn','session','completed',1,2,'reply')")
            .execute(&pool).await.unwrap();
        pool
    }

    fn output(id: &str, kind: &str) -> PreparedOutput {
        PreparedOutput {
            node: InsertNode {
                id: id.into(),
                kind: kind.into(),
                initial_x: 0.0,
                initial_y: 0.0,
                width: 260.0,
                height: 184.0,
                sizing_mode: structure::SizingMode::Content,
                appearance: None,
                file_subpath: None,
                data: json!({"kind":kind,"title":"Report","text":"Findings"}),
                source_chunks: Vec::new(),
            },
        }
    }

    #[tokio::test]
    async fn publishes_nodes_message_and_run_in_one_commit() {
        let pool = fixture().await;
        commit(
            &pool,
            "session",
            "run",
            "space",
            &[output("report", "markdown")],
            None,
        )
        .await
        .unwrap();
        let result: (String, String, String, i64) = sqlx::query_as(
            "SELECT r.state,m.content,n.kind,e.revision FROM spaces_runs r
             JOIN spaces_assistant_messages m ON m.id='reply'
             JOIN spaces_nodes n ON n.id='report'
             JOIN spaces_edit_state e ON e.space_id='space' WHERE r.id='run'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(result.0, "succeeded");
        assert_eq!(result.2, "markdown");
        assert_eq!(result.3, 1);
        let message: Value = serde_json::from_str(&result.1).unwrap();
        assert_eq!(message["outputs"][0]["nodeId"], "report");
    }

    #[tokio::test]
    async fn invalid_output_rolls_back_all_publication_state() {
        let pool = fixture().await;
        assert!(commit(
            &pool,
            "session",
            "run",
            "space",
            &[output("good", "markdown"), output("bad", "invalid")],
            None
        )
        .await
        .is_err());
        let node_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_nodes")
            .fetch_one(&pool)
            .await
            .unwrap();
        let state: String = sqlx::query_scalar("SELECT state FROM spaces_runs WHERE id='run'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(node_count, 0);
        assert_eq!(state, "running");
    }

    #[tokio::test]
    async fn existing_agent_created_node_is_attached_without_duplicate_import() {
        let pool = fixture().await;
        let mut tx = db::begin_write(&pool).await.unwrap();
        let mut node = output("mcp-note", "markdown").node;
        knowledge::contracts::normalize_node(&mut tx, "space", &node.kind, &mut node.data)
            .await
            .unwrap();
        let (mut snapshot, revision) = structure::repository::load_in_transaction(&mut tx, "space")
            .await
            .unwrap();
        structure::creation::append_content(&mut snapshot, std::slice::from_ref(&node), None)
            .unwrap();
        nodes::insert_rows_with_actor(
            &mut tx,
            "space",
            std::slice::from_ref(&node),
            knowledge::events::EventActor::new("agent", Some("session")),
            2,
            None,
        )
        .await
        .unwrap();
        structure::repository::save_in_transaction(&mut tx, "space", revision, &snapshot)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        assert!(super::super::repository::has_turn_output(&pool, "turn")
            .await
            .unwrap());
        commit(&pool, "session", "run", "space", &[], None)
            .await
            .unwrap();
        let content: String =
            sqlx::query_scalar("SELECT content FROM spaces_assistant_messages WHERE id='reply'")
                .fetch_one(&pool)
                .await
                .unwrap();
        let content: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(content["outputs"][0]["nodeId"], "mcp-note");
        let node_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_nodes")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(node_count, 1);
    }

    #[tokio::test]
    async fn context_attachment_does_not_assert_derivation() {
        let pool = fixture().await;
        let mut tx = db::begin_write(&pool).await.unwrap();
        nodes::insert_many_in_transaction(
            &mut tx,
            nodes::contracts::InsertNodesRequest {
                space_id: "space".into(),
                mutation_id: "source-create".into(),
                expected_edit_revision: 0,
                target_parent_id: None,
                nodes: vec![output("source", "markdown").node],
            },
            None,
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();
        sqlx::query("UPDATE spaces_runs SET input_snapshot=$1 WHERE id='run'")
            .bind(json!({"attachments":[{"source":"node","nodeId":"source"}]}).to_string())
            .execute(&pool)
            .await
            .unwrap();
        commit(
            &pool,
            "session",
            "run",
            "space",
            &[output("derived", "markdown")],
            None,
        )
        .await
        .unwrap();
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM spaces_relations WHERE space_id='space'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn cancel_request_prevents_publication() {
        let pool = fixture().await;
        sqlx::query("UPDATE spaces_runs SET cancel_requested_at=3 WHERE id='run'")
            .execute(&pool)
            .await
            .unwrap();
        assert!(commit(
            &pool,
            "session",
            "run",
            "space",
            &[output("late", "markdown")],
            None
        )
        .await
        .is_err());
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_nodes")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn disconnected_session_keeps_completed_turn_outputs_retryable() {
        let pool = fixture().await;
        super::super::runtime::mark_disconnected(&pool, "session", "run", "ACP finish failed")
            .await
            .unwrap();
        commit(
            &pool,
            "session",
            "run",
            "space",
            &[output("report", "markdown")],
            None,
        )
        .await
        .unwrap();
        let state: String = sqlx::query_scalar("SELECT state FROM spaces_runs WHERE id='run'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(state, "succeeded");
    }
    #[tokio::test]
    async fn publication_failure_is_retryable_and_receipts_are_released_atomically() {
        let pool = fixture().await;
        sqlx::query("UPDATE spaces_runs SET state='failed',completed_at=3 WHERE id='run'")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("UPDATE spaces_external_agent_conversations SET output_publication_state='failed',output_publication_error='Disk full' WHERE id='session'")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO spaces_external_output_staging(run_id,path,role,created_at) VALUES('run','/managed/report','asset',1)")
            .execute(&pool).await.unwrap();
        commit(
            &pool,
            "session",
            "run",
            "space",
            &[output("report", "markdown")],
            None,
        )
        .await
        .unwrap();
        let state: (String, String, i64) = sqlx::query_as(
            "SELECT r.state,c.output_publication_state,(SELECT COUNT(*) FROM spaces_external_output_staging) FROM spaces_runs r JOIN spaces_external_agent_conversations c ON c.run_id=r.id WHERE r.id='run'")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(state, ("succeeded".into(), "committed".into(), 0));
        assert!(commit(
            &pool,
            "session",
            "run",
            "space",
            &[output("duplicate", "markdown")],
            None
        )
        .await
        .is_err());
        let actor: String = sqlx::query_scalar("SELECT actor_id FROM spaces_knowledge_events WHERE entity_id='report' AND event_kind='created'")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(actor, "session");
    }

    #[tokio::test]
    async fn unfinished_children_block_publication_and_failed_children_are_visible() {
        let pool = fixture().await;
        sqlx::query("INSERT INTO spaces_runs(id,home_space_id,parent_run_id,principal_kind,principal_id,kind,state,policy_snapshot,input_snapshot,budget,created_at,updated_at) VALUES('child','space','run','external_agent','session','action','running','{}','{}','{}',1,1)")
            .execute(&pool).await.unwrap();
        assert!(commit(
            &pool,
            "session",
            "run",
            "space",
            &[output("too-early", "markdown")],
            None
        )
        .await
        .is_err());
        sqlx::query("UPDATE spaces_runs SET state='failed',completed_at=3 WHERE id='child'")
            .execute(&pool)
            .await
            .unwrap();
        commit(
            &pool,
            "session",
            "run",
            "space",
            &[output("report", "markdown")],
            None,
        )
        .await
        .unwrap();
        let state: String = sqlx::query_scalar("SELECT state FROM spaces_runs WHERE id='run'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(state, "completed_with_errors");
    }

    #[tokio::test]
    async fn cancelling_run_immediately_cancels_all_descendants() {
        let pool = fixture().await;
        for (id, parent) in [("child", "run"), ("grandchild", "child")] {
            sqlx::query("INSERT INTO spaces_runs(id,home_space_id,parent_run_id,principal_kind,principal_id,kind,state,policy_snapshot,input_snapshot,budget,created_at,updated_at) VALUES($1,'space',$2,'external_agent','session','action','running','{}','{}','{}',1,1)")
                .bind(id).bind(parent).execute(&pool).await.unwrap();
        }
        crate::spaces::runs::request_cancellation(&pool, "run")
            .await
            .unwrap();
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_runs WHERE cancel_requested_at IS NOT NULL AND execution_epoch=1")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(count, 3);
        assert!(commit(&pool, "session", "run", "space", &[], None)
            .await
            .is_err());
    }
}
