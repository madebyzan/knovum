use super::super::record_usage;

async fn database() -> sqlx::SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    sqlx::raw_sql("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1);
        INSERT INTO spaces_runs(id,scope_kind,home_space_id,principal_kind,principal_id,kind,state,policy_snapshot,input_snapshot,budget,created_at,updated_at)
        VALUES('parent','space','space','user','user','builtin_agent','running','{}','{}','{\"maxTotalTokens\":100}',1,1);
        INSERT INTO spaces_runs(id,scope_kind,home_space_id,parent_run_id,principal_kind,principal_id,kind,state,policy_snapshot,input_snapshot,budget,created_at,updated_at)
        VALUES('child','space','space','parent','user','user','action','running','{}','{}','{}',1,1);")
        .execute(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn a_child_receipt_is_recorded_on_its_cancelled_ancestors() {
    let pool = database().await;
    sqlx::query("UPDATE spaces_runs SET state='cancelled',completed_at=2,cancel_requested_at=2")
        .execute(&pool)
        .await
        .unwrap();
    record_usage(&pool, "child", 20, 0, 20).await.unwrap();
    let totals: Vec<i64> = sqlx::query_scalar("SELECT total_tokens FROM spaces_runs ORDER BY id")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(totals, vec![20, 20]);
}

#[tokio::test]
async fn budget_exhaustion_cannot_roll_back_already_paid_child_usage() {
    let pool = database().await;
    assert!(record_usage(&pool, "child", 200, 0, 200)
        .await
        .unwrap_err()
        .starts_with("budget-exhausted"));
    let totals: Vec<i64> = sqlx::query_scalar("SELECT total_tokens FROM spaces_runs ORDER BY id")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(totals, vec![200, 200]);
}
