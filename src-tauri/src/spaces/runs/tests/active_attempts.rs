use super::*;

async fn claim(pool: &sqlx::SqlitePool, parent: Option<&str>) -> AttemptClaim {
    let run_id = uuid::Uuid::new_v4().to_string();
    let step_instance_id = uuid::Uuid::new_v4().to_string();
    let attempt_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO spaces_runs(id,home_space_id,parent_run_id,principal_kind,principal_id,kind,state,policy_snapshot,input_snapshot,budget,created_at,updated_at) VALUES(?,'space',?,'user','user','workflow','running','{}','{}','{}',1,1)")
        .bind(&run_id).bind(parent).execute(pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_run_steps(id,run_id,definition_node_id,instance_path,node_kind,state,resolved_input,created_at,updated_at) VALUES(?,?,'segment','segment','speech_segment','running','{}',1,1)")
        .bind(&step_instance_id).bind(&run_id).execute(pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_run_attempts(id,run_id,step_instance_id,attempt_number,state,request_digest,executor_id,idempotency_key,execution_epoch,lease_until,started_at) VALUES(?,?,?,1,'executing',?,'desktop',?,0,?,1)")
        .bind(&attempt_id).bind(&run_id).bind(&step_instance_id).bind("a".repeat(64))
        .bind(&attempt_id).bind(crate::clock::unix_time_millis().unwrap()+60_000)
        .execute(pool).await.unwrap();
    AttemptClaim {
        attempt_id,
        run_id,
        step_instance_id,
        execution_epoch: 0,
        attempt_number: 1,
        home_space_id: "space".into(),
        node_kind: "speech_segment".into(),
        configuration: serde_json::json!({}),
        resolved_input: serde_json::json!({}),
        policy_snapshot: serde_json::json!({}),
        budget: serde_json::json!({}),
        cancellation: CancellationToken::new(),
    }
}

#[tokio::test]
async fn cancellation_interrupts_descendants_and_registration_races_without_waiting_for_renewal() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
        .execute(&pool).await.unwrap();
    let root = claim(&pool, None).await;
    let child = claim(&pool, Some(&root.run_id)).await;
    let unrelated = claim(&pool, None).await;
    let _root = register(&root).unwrap();
    let _child = register(&child).unwrap();
    let _unrelated = register(&unrelated).unwrap();
    super::super::request_cancellation(&pool, &root.run_id)
        .await
        .unwrap();
    assert!(root.cancellation.is_cancelled());
    assert!(child.cancellation.is_cancelled());
    assert!(!unrelated.cancellation.is_cancelled());

    // Cancellation can commit between claiming and registering the local worker.
    let late = claim(&pool, None).await;
    super::super::request_cancellation(&pool, &late.run_id)
        .await
        .unwrap();
    let _late = register(&late).unwrap();
    reconcile(&pool).await.unwrap();
    assert!(late.cancellation.is_cancelled());
    assert!(!unrelated.cancellation.is_cancelled());
    sqlx::query("UPDATE spaces_runs SET execution_epoch=execution_epoch+1 WHERE id=?")
        .bind(&unrelated.run_id)
        .execute(&pool)
        .await
        .unwrap();
    reconcile(&pool).await.unwrap();
    assert!(unrelated.cancellation.is_cancelled());
}
