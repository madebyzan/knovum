use super::fixture;
use crate::{
    ai::{
        models::{ModelExecutionTarget, ModelTargetSelection},
        settings::contracts::BuiltinAgentModelSettings,
    },
    spaces::conversations::execution::session,
};

#[tokio::test]
async fn successful_followup_reuses_binding_and_real_catalog_changes_create_a_new_segment() {
    let pool = fixture::database().await;
    let ids = fixture::ids();
    let target = ModelExecutionTarget {
        provider: "openai".into(),
        model: "model".into(),
        base_url: "https://example.test/v1".into(),
        temperature: 0.2,
        secret_key: "credential-reference".into(),
        capabilities: vec![],
    };
    let profile = BuiltinAgentModelSettings {
        context_window: 65_536,
        max_output_tokens: 4096,
    };
    let selection = ModelTargetSelection::Model {
        connection_id: "connection".into(),
        model_id: "model".into(),
        temperature: 0.2,
    };
    let catalog = crate::spaces::conversations::tools::project().unwrap();
    let mut tx = pool.begin().await.unwrap();
    let first = session::select(
        &mut tx,
        &ids,
        &target,
        &profile,
        &selection,
        &catalog.digest,
        2,
    )
    .await
    .unwrap();
    sqlx::query("UPDATE spaces_builtin_agent_tasks SET segment_id=$1 WHERE id='task'")
        .bind(&first.thread_id)
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query("UPDATE spaces_runs SET state='succeeded',completed_at=2 WHERE id='run'")
        .execute(&mut *tx)
        .await
        .unwrap();
    let followup = session::select(
        &mut tx,
        &ids,
        &target,
        &profile,
        &selection,
        &catalog.digest,
        3,
    )
    .await
    .unwrap();
    assert_eq!(first.thread_id, followup.thread_id);
    assert_eq!(first.execution_revision, followup.execution_revision);
    let changed = session::select(
        &mut tx,
        &ids,
        &target,
        &profile,
        &selection,
        &"b".repeat(64),
        4,
    )
    .await
    .unwrap();
    assert_ne!(changed.thread_id, first.thread_id);
    tx.commit().await.unwrap();
}
