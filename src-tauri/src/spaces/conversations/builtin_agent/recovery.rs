use tauri::AppHandle;

use crate::spaces::conversations::{
    execution::gateway::Gateway, repository, tools, StartBuiltinAgentTaskRequest,
};

pub(super) async fn recover_all(app: AppHandle) -> Result<(), String> {
    let pool = crate::spaces::db::pool(&app).await?;
    super::incompatible_segments::retire(&app, pool).await?;
    for task in repository::recoverable_tasks(pool).await? {
        let run = crate::spaces::runs::load_run(pool, &task.run_id).await?;
        if matches!(
            run.wait_reason,
            Some(
                crate::spaces::runs::WaitReason::ToolApproval
                    | crate::spaces::runs::WaitReason::ToolOutcomeUnknown
            )
        ) {
            continue;
        }
        recover_task(&app, pool, task, false, None).await?;
    }
    Ok(())
}

pub(super) async fn recover_one(
    app: AppHandle,
    run_id: &str,
    resume_interrupt: bool,
    expected_retry_version: Option<i64>,
) -> Result<(), String> {
    let pool = crate::spaces::db::pool(&app).await?;
    super::incompatible_segments::retire(&app, pool).await?;
    let task = repository::recoverable_task(pool, run_id).await?;
    recover_task(&app, pool, task, resume_interrupt, expected_retry_version).await
}

async fn recover_task(
    app: &AppHandle,
    pool: &sqlx::SqlitePool,
    task: repository::RecoverableTask,
    user_resume: bool,
    expected_retry_version: Option<i64>,
) -> Result<(), String> {
    if task.profile_version != super::BUILTIN_AGENT_PROFILE_VERSION {
        return Err("builtin-agent-profile-changed: start a new execution segment".into());
    }
    crate::extensions::snapshots::require_current(app, &task.extensions).await?;
    let mut catalog = tools::project()?;
    catalog.extend(&task.extensions)?;
    if catalog.digest != task.catalog_digest {
        return Err("builtin-agent-action-catalog-changed: start a new execution segment".into());
    }
    let request = StartBuiltinAgentTaskRequest {
        space_id: task.space_id,
        conversation_id: Some(task.conversation_id.clone()),
        branch_id: Some(task.branch_id.clone()),
        prompt: task.prompt,
        target: task.model_binding,
        max_turns: task.max_turns,
        max_tool_calls: task.max_tool_calls,
        max_duration_seconds: task.max_duration_seconds,
        attachments: Vec::new(),
        extensions: Default::default(),
    };
    let ids = super::identity::TaskIds {
        task_id: task.task_id,
        run_id: task.run_id,
        conversation_id: task.conversation_id,
        branch_id: task.branch_id,
        grant_id: task.grant_id,
    };
    let mut session = task.session;
    let cancellation = super::cancellation::register(&ids.run_id)?;
    let prepared = prepare(
        pool,
        &ids.task_id,
        &ids.run_id,
        user_resume,
        expected_retry_version,
    )
    .await;
    match prepared {
        Ok(epoch) => session.control_epoch = epoch,
        Err(error) => {
            super::cancellation::remove(&ids.run_id);
            return Err(error);
        }
    }
    crate::spaces::runs::announce_change();
    let (messages, resume) = if user_resume && session.projection_checkpoint_id.is_some() {
        (Vec::new(), Some(serde_json::json!(true)))
    } else {
        (task.messages, None)
    };
    super::lifecycle::launch(
        Gateway {
            app: app.clone(),
            pool: pool.clone(),
            target: task.frozen_target,
            profile: task.model_profile,
            request,
            ids,
            catalog,
            extensions: task.extensions,
            session,
            updates: None,
            last_message: std::sync::Mutex::new(None),
        },
        messages,
        resume,
        cancellation,
    );
    Ok(())
}

pub(super) async fn prepare(
    pool: &sqlx::SqlitePool,
    task_id: &str,
    run_id: &str,
    user_resume: bool,
    expected_retry_version: Option<i64>,
) -> Result<u64, String> {
    let mut tx = crate::spaces::db::begin_write(pool).await?;
    let active: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_builtin_agent_tasks t JOIN spaces_runs r ON r.id=t.run_id
        WHERE r.id!=$1 AND r.completed_at IS NULL AND r.state IN ('queued','running','waiting')",
    )
    .bind(run_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(crate::spaces::db::sql_error)?;
    if active != 0 {
        return Err("builtin-agent-busy: another built-in execution is active".into());
    }
    let pending: i64=sqlx::query_scalar("SELECT count(*) FROM spaces_conversation_tool_operations WHERE run_id=$1 AND ((state='prepared' AND approval IS NULL) OR (state='outcome_unknown' AND resolution IS NULL))").bind(run_id).fetch_one(&mut *tx).await.map_err(crate::spaces::db::sql_error)?;
    if pending != 0 {
        return Err("builtin-agent-tool-intervention-required".into());
    }
    let now = crate::foundation::clock::unix_time_millis()?;
    if let Some(version) = expected_retry_version {
        super::model_recovery::authorize_retry(&mut tx, run_id, version, now).await?;
    }
    let sql = if user_resume {
        "UPDATE spaces_runs SET state='running',wait_reason=NULL,pause_requested_at=NULL,state_version=state_version+1,updated_at=$1
         WHERE id=$2 AND completed_at IS NULL AND cancel_requested_at IS NULL AND ((state='waiting' AND pause_requested_at IS NULL AND wait_reason IN ('tool_approval','tool_outcome_unknown')) OR (state='paused' AND pause_requested_at IS NOT NULL
         AND EXISTS(SELECT 1 FROM spaces_run_events e WHERE e.run_id=spaces_runs.id AND e.kind='builtin_agent_paused_checkpoint' AND e.created_at>=spaces_runs.pause_requested_at)))"
    } else {
        "UPDATE spaces_runs SET state='running',wait_reason=NULL,state_version=state_version+1,updated_at=$1
         WHERE id=$2 AND state IN ('queued','running','waiting') AND completed_at IS NULL AND pause_requested_at IS NULL AND cancel_requested_at IS NULL"
    };
    let updated = sqlx::query(sql)
        .bind(now)
        .bind(run_id)
        .execute(&mut *tx)
        .await
        .map_err(crate::spaces::db::sql_error)?;
    crate::spaces::db::require_one(
        updated,
        "builtin-agent-recovery-state-changed: execution or paused checkpoint is unavailable",
    )?;
    let epoch: i64 = sqlx::query_scalar("UPDATE spaces_builtin_agent_tasks SET control_epoch=control_epoch+1 WHERE id=$1 RETURNING control_epoch")
        .bind(task_id).fetch_one(&mut *tx).await.map_err(crate::spaces::db::sql_error)?;
    let epoch = u64::try_from(epoch).map_err(|_| "builtin-agent-control-epoch-invalid")?;
    tx.commit().await.map_err(crate::spaces::db::sql_error)?;
    Ok(epoch)
}
