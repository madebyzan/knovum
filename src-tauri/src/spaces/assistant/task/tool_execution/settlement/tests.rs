
use super::*;

#[tokio::test]
async fn accepted_child_settles_while_its_assistant_parent_is_waiting() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_runs(id,home_space_id,principal_kind,principal_id,kind,state,policy_snapshot,input_snapshot,budget,created_at,updated_at) VALUES('parent','space','assistant','principal','assistant','running','{}','{}','{}',1,1)").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_runs(id,home_space_id,parent_run_id,principal_kind,principal_id,kind,state,completed_at,policy_snapshot,input_snapshot,budget,created_at,updated_at) VALUES('child','space','parent','assistant','principal','action','succeeded',2,'{}','{}','{}',1,2)").execute(&pool).await.unwrap();
    let result = settle(
        &pool,
        "parent",
        ActionInvocationOutcome::AcceptedRun {
            run_id: "child".into(),
        },
    )
    .await
    .unwrap();
    assert_eq!(result["state"], "succeeded");
    sqlx::query("UPDATE spaces_runs SET cancel_requested_at=3,execution_epoch=execution_epoch+1 WHERE id='parent'")
            .execute(&pool).await.unwrap();
    assert!(set_execution_state(&pool, "parent", "running", None)
        .await
        .is_err());
    assert_eq!(
        crate::spaces::runs::load_run(&pool, "parent")
            .await
            .unwrap()
            .state,
        RunState::Running
    );
}
