use super::action_pool;
use crate::spaces::{db, derivation::failure::fail_messages};

async fn pending(pool: &sqlx::SqlitePool) {
    for id in ["first", "second"] {
        sqlx::query("INSERT INTO spaces_runs(id,scope_kind,home_space_id,principal_kind,principal_id,kind,state,execution_epoch,policy_snapshot,input_snapshot,budget,created_at,updated_at) VALUES($1,'space','space','user','local-user','action','running',0,'{}','{}','{}',1,1)")
            .bind(id).execute(pool).await.unwrap();
        sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES($1,'space','image',0,0,json_object('id',$1,'kind','image','title','Layer','processingState','processing'),1,1)")
            .bind(id).execute(pool).await.unwrap();
        sqlx::query("INSERT INTO spaces_knowledge_derivations(id,space_id,output_node_id,action_kind,action_version,engine,status,created_at,updated_at) VALUES($1,'space',$1,'split-image-layers',1,'test','running',1,1)")
            .bind(id).execute(pool).await.unwrap();
        sqlx::query("INSERT INTO spaces_reserved_outputs(node_id,space_id,run_id,state_field,created_at) VALUES($1,'space',$1,'processingState',1)")
            .bind(id).execute(pool).await.unwrap();
    }
}

#[tokio::test]
async fn settles_all_outputs_in_one_transaction() {
    let pool = action_pool().await;
    pending(&pool).await;
    fail_messages(
        &pool,
        &["first".into(), "second".into()],
        "generation failed",
    )
    .await
    .unwrap();
    let states: Vec<(String, String)> = sqlx::query_as("SELECT d.status,json_extract(n.data,'$.processingState') FROM spaces_knowledge_derivations d JOIN spaces_nodes n ON n.id=d.output_node_id ORDER BY d.id")
        .fetch_all(&pool).await.unwrap();
    assert_eq!(states, vec![("failed".into(), "failed".into()); 2]);
}

#[tokio::test]
async fn settlement_failure_rolls_back_every_output() {
    let pool = action_pool().await;
    pending(&pool).await;
    sqlx::query("CREATE TRIGGER reject_second_failure BEFORE UPDATE ON spaces_knowledge_derivations WHEN OLD.id='second' BEGIN SELECT RAISE(ABORT,'settlement unavailable'); END")
        .execute(&pool).await.unwrap();
    assert!(fail_messages(
        &pool,
        &["first".into(), "second".into()],
        "generation failed"
    )
    .await
    .unwrap_err()
    .contains("settlement unavailable"));
    let mut tx = db::begin_write(&pool).await.unwrap();
    let running: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_knowledge_derivations WHERE status='running'",
    )
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    assert_eq!(running, 2);
}

#[tokio::test]
async fn database_settlement_failure_retains_the_producer_error_code_and_both_causes() {
    let pool = action_pool().await;
    pending(&pool).await;
    sqlx::query("CREATE TRIGGER reject_failure BEFORE UPDATE ON spaces_knowledge_derivations BEGIN SELECT RAISE(ABORT,'settlement unavailable'); END")
        .execute(&pool).await.unwrap();
    let primary = crate::foundation::app_error::AppError::new(
        "provider_request_failed",
        "generation rejected",
    );
    let settlement = fail_messages(&pool, &["first".into()], &primary.message).await;
    let error = super::super::failure::with_settlement(primary, settlement);
    assert_eq!(error.code, "provider_request_failed");
    assert!(error.message.contains("generation rejected"));
    assert!(error.message.contains("settlement unavailable"));
    let state: String =
        sqlx::query_scalar("SELECT status FROM spaces_knowledge_derivations WHERE id='first'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(state, "running");
}
