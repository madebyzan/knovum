mod failure;
mod image_tensors;

#[tokio::test]
async fn stale_claim_cannot_adopt_the_current_execution_epoch() {
    let pool = action_pool().await;
    sqlx::query("INSERT INTO spaces_runs(id,scope_kind,home_space_id,principal_kind,principal_id,kind,state,execution_epoch,policy_snapshot,input_snapshot,budget,created_at,updated_at) VALUES('action','space','space','builtin_agent','task','action','running',7,'{}','{}','{}',1,1)")
        .execute(&pool)
        .await
        .unwrap();
    let stale = super::action_execution::ExecutionAuthority {
        run_id: "action".into(),
        execution_epoch: 7,
    };
    stale.require_active(&pool).await.unwrap();

    crate::spaces::runs::request_cancellation(&pool, "action")
        .await
        .unwrap();
    sqlx::query(
        "UPDATE spaces_runs SET cancel_requested_at=NULL,state='running' WHERE id='action'",
    )
    .execute(&pool)
    .await
    .unwrap();

    assert!(stale
        .require_active(&pool)
        .await
        .unwrap_err()
        .contains("stale"));
}

#[tokio::test]
async fn pending_pause_keeps_the_claimed_action_authority_active() {
    let pool = action_pool().await;
    sqlx::query("INSERT INTO spaces_runs(id,scope_kind,home_space_id,principal_kind,principal_id,kind,state,execution_epoch,pause_requested_at,policy_snapshot,input_snapshot,budget,created_at,updated_at) VALUES('paused-action','space','space','builtin_agent','task','action','running',3,2,'{}','{}','{}',1,2)")
        .execute(&pool)
        .await
        .unwrap();
    let authority = super::action_execution::ExecutionAuthority {
        run_id: "paused-action".into(),
        execution_epoch: 3,
    };
    authority.require_active(&pool).await.unwrap();
    crate::spaces::runs::request_cancellation(&pool, "paused-action")
        .await
        .unwrap();
    assert!(authority.require_active(&pool).await.is_err());
}

#[tokio::test]
async fn cancelled_authority_cannot_reserve_or_publish() {
    let pool = action_pool().await;
    sqlx::query("CREATE TABLE guarded_writes(phase TEXT NOT NULL)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO spaces_runs(id,scope_kind,home_space_id,principal_kind,principal_id,kind,state,execution_epoch,policy_snapshot,input_snapshot,budget,created_at,updated_at) VALUES('guarded','space','space','builtin_agent','task','action','running',2,'{}','{}','{}',1,1)")
        .execute(&pool)
        .await
        .unwrap();
    let authority = super::action_execution::ExecutionAuthority {
        run_id: "guarded".into(),
        execution_epoch: 2,
    };
    crate::spaces::runs::request_cancellation(&pool, "guarded")
        .await
        .unwrap();

    for phase in ["pending-reservation", "generation-publication"] {
        assert!(guarded_write(&pool, &authority, phase).await.is_err());
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM guarded_writes WHERE phase=$1")
            .bind(phase)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }
}

async fn guarded_write(
    pool: &sqlx::SqlitePool,
    authority: &super::action_execution::ExecutionAuthority,
    phase: &str,
) -> Result<(), String> {
    let mut tx = crate::spaces::db::begin_write(pool).await?;
    if let Err(error) = authority.require_active(&mut *tx).await {
        tx.rollback().await.map_err(crate::spaces::db::sql_error)?;
        return Err(error);
    }
    sqlx::query("INSERT INTO guarded_writes(phase) VALUES($1)")
        .bind(phase)
        .execute(&mut *tx)
        .await
        .map_err(crate::spaces::db::sql_error)?;
    tx.commit().await.map_err(crate::spaces::db::sql_error)
}

async fn action_pool() -> sqlx::SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at,last_opened_at) VALUES('space','space','Space',1,1,1)")
        .execute(&pool)
        .await
        .unwrap();
    pool
}
