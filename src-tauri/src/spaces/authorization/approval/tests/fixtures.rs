use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

use super::super::{create, ApprovalRequest};
use crate::spaces::{
    authorization::{create_grant, AuthorizationContext, Principal, PrincipalKind},
    db,
};

pub(super) async fn approved() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("database");
    db::migrate(&pool).await.expect("current schema");
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)").execute(&pool).await.expect("space");
    let mut tx = db::begin_write(&pool).await.expect("begin");
    for id in ["grant-a", "grant-b"] {
        create_grant(
            &mut tx,
            &AuthorizationContext {
                principal: Principal {
                    kind: PrincipalKind::Assistant,
                    id: id.into(),
                },
                grant_id: id.into(),
                home_space_id: "space".into(),
                read_space_ids: vec!["space".into()],
                write_space_ids: vec!["space".into()],
                action_ids: vec!["action".into()],
                valid_until: i64::MAX,
            },
            1,
        )
        .await
        .expect("grant");
    }
    create(
        &mut tx,
        &ApprovalRequest {
            id: "approval".into(),
            grant_id: "grant-a".into(),
            space_id: "space".into(),
            action_id: "action".into(),
            plan_digest: "a".repeat(64),
            plan: serde_json::json!({}),
            expected_revisions: serde_json::json!({}),
            valid_until: 100,
        },
        1,
    )
    .await
    .expect("approval");
    sqlx::query("UPDATE spaces_action_approvals SET decision='approved' WHERE id='approval'")
        .execute(&mut *tx)
        .await
        .expect("approve");
    tx.commit().await.expect("commit");
    pool
}
