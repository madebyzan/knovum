mod fixtures;

use super::consume;
use crate::spaces::db;
use fixtures::approved;

#[tokio::test]
async fn rejects_another_grant_without_consuming_the_approval() {
    let pool = approved().await;
    let mut tx = db::begin_write(&pool).await.expect("begin");
    let error = consume(
        &mut tx,
        "approval",
        "grant-b",
        "space",
        "action",
        &"a".repeat(64),
        10,
    )
    .await
    .expect_err("cross-grant approval must fail");
    assert!(error.contains("authorization context"));
    consume(
        &mut tx,
        "approval",
        "grant-a",
        "space",
        "action",
        &"a".repeat(64),
        10,
    )
    .await
    .expect("owner approval still usable");
    consume(
        &mut tx,
        "approval",
        "grant-a",
        "space",
        "action",
        &"a".repeat(64),
        10,
    )
    .await
    .expect_err("one consumption");
}

#[tokio::test]
async fn rejects_space_plan_action_and_expiry_changes() {
    let pool = approved().await;
    let mut tx = db::begin_write(&pool).await.expect("begin");
    for (space, action, digest, now) in [
        ("other", "action", "a".repeat(64), 10),
        ("space", "other", "a".repeat(64), 10),
        ("space", "action", "b".repeat(64), 10),
        ("space", "action", "a".repeat(64), 101),
    ] {
        consume(&mut tx, "approval", "grant-a", space, action, &digest, now)
            .await
            .expect_err("changed context must fail");
    }
    consume(
        &mut tx,
        "approval",
        "grant-a",
        "space",
        "action",
        &"a".repeat(64),
        100,
    )
    .await
    .expect("valid until inclusive");
}

#[tokio::test]
async fn rejected_or_pending_approval_cannot_be_consumed() {
    let pool = approved().await;
    let mut tx = db::begin_write(&pool).await.expect("begin");
    for decision in ["rejected", "pending"] {
        sqlx::query("UPDATE spaces_action_approvals SET decision=$1 WHERE id='approval'")
            .bind(decision)
            .execute(&mut *tx)
            .await
            .expect("decision");
        consume(
            &mut tx,
            "approval",
            "grant-a",
            "space",
            "action",
            &"a".repeat(64),
            10,
        )
        .await
        .expect_err("not approved");
    }
}

#[tokio::test]
async fn concurrent_consumers_commit_exactly_once() {
    let pool = approved().await;
    async fn attempt(pool: &sqlx::SqlitePool) -> Result<(), String> {
        let mut tx = db::begin_write(pool).await?;
        consume(
            &mut tx,
            "approval",
            "grant-a",
            "space",
            "action",
            &"a".repeat(64),
            10,
        )
        .await?;
        tx.commit().await.map_err(db::sql_error)
    }
    let (first, second) = tokio::join!(attempt(&pool), attempt(&pool));
    assert_eq!(usize::from(first.is_ok()) + usize::from(second.is_ok()), 1);
    let consumed: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_action_approvals WHERE consumed_at IS NOT NULL",
    )
    .fetch_one(&pool)
    .await
    .expect("consumed approvals");
    assert_eq!(consumed, 1);
}
