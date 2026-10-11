use sqlx::SqliteConnection;

pub(crate) fn reserve_maintenance(
) -> Result<[crate::foundation::job_registry::JobAdmissionReservation; 4], String> {
    let packages = super::package::control::reserve_maintenance()?;
    let [imports, media] = super::assets::import::reserve_maintenance()?;
    let canvas = super::canvas_exchange::reserve_maintenance()?;
    Ok([packages, imports, media, canvas])
}

pub(crate) async fn prepare_package_staging<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<(), String> {
    super::package::reconcile(app)
        .await
        .map_err(|error| format!("spaces-backup-not-quiescent: {error}"))
}

/// Rejects a snapshot that could resume or reconcile Spaces-owned side effects after restore.
/// Unsent conversation drafts are durable local data and do not prevent a snapshot.
pub(crate) async fn assert_quiescent(connection: &mut SqliteConnection) -> Result<(), String> {
    let reason: Option<String> = sqlx::query_scalar(
        r#"SELECT reason FROM (
           SELECT 1 AS priority,'non-terminal Run' AS reason WHERE EXISTS(
             SELECT 1 FROM spaces_runs
             WHERE completed_at IS NULL
                OR state NOT IN ('succeeded','completed_with_errors','failed','cancelled'))
           UNION ALL SELECT 2,'non-terminal Run step' WHERE EXISTS(
             SELECT 1 FROM spaces_run_steps
             WHERE state NOT IN ('succeeded','failed','skipped','blocked','cancelled'))
           UNION ALL SELECT 12,'queued or active knowledge preparation' WHERE EXISTS(
             SELECT 1 FROM spaces_enrichment_jobs
             WHERE status IN ('pending','running','awaiting_configuration'))
           UNION ALL SELECT 13,'active knowledge derivation' WHERE EXISTS(
             SELECT 1 FROM spaces_knowledge_derivations WHERE status='running')
           UNION ALL SELECT 3,'active Run attempt lease' WHERE EXISTS(
             SELECT 1 FROM spaces_run_attempts
             WHERE state IN ('claimed','executing','waiting_external'))
           UNION ALL SELECT 4,'active Agent model call' WHERE EXISTS(
             SELECT 1 FROM spaces_builtin_agent_model_calls WHERE state='requesting')
           UNION ALL SELECT 5,'unsettled tool operation' WHERE EXISTS(
             SELECT 1 FROM spaces_conversation_tool_operations
             WHERE state NOT IN ('completed','failed')
               AND NOT (state='outcome_unknown' AND resolution IS NOT NULL))
           UNION ALL SELECT 6,'pending tool approval or elicitation' WHERE EXISTS(
             SELECT 1 FROM spaces_conversation_mcp_interactions WHERE response IS NULL)
           UNION ALL SELECT 7,'active External Agent' WHERE EXISTS(
             SELECT 1 FROM spaces_external_agent_conversations
             WHERE state IN ('connecting','connected'))
             OR EXISTS(SELECT 1 FROM spaces_external_agent_turns WHERE state='running')
           UNION ALL SELECT 8,'recoverable External Agent publication' WHERE EXISTS(
             SELECT 1 FROM spaces_external_agent_conversations c
             JOIN spaces_runs r ON r.id=c.run_id
             WHERE c.output_publication_state<>'committed'
               AND r.cancel_requested_at IS NULL AND r.state<>'cancelled'
               AND EXISTS(SELECT 1 FROM spaces_external_agent_turns t
                          WHERE t.conversation_id=c.id AND t.state='completed'))
           UNION ALL SELECT 9,'staged External Agent output' WHERE EXISTS(
             SELECT 1 FROM spaces_external_output_staging)
           UNION ALL SELECT 10,'unpublished speech asset owner work' WHERE EXISTS(
             SELECT 1 FROM spaces_speech_asset_publications)
           UNION ALL SELECT 11,'staged speech render' WHERE EXISTS(
             SELECT 1 FROM spaces_speech_renders WHERE state='staged')
         ) blockers ORDER BY priority LIMIT 1"#,
    )
    .fetch_optional(&mut *connection)
    .await
    .map_err(super::db::sql_error)?;
    match reason {
        Some(reason) => Err(format!(
            "spaces-backup-not-quiescent: Spaces contains {reason}"
        )),
        None => Ok(()),
    }
}
