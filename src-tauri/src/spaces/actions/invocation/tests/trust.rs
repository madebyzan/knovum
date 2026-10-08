use serde_json::{json, Value};
use sqlx::sqlite::SqlitePoolOptions;

use super::super::{invoke, ActionInvocationOutcome, ActionInvocationRequest};
use crate::spaces::{
    authorization::{create_grant, AuthorizationContext, Principal, PrincipalKind},
    db,
};

#[tokio::test]
async fn both_agent_principals_create_canvas_resources_without_approval_and_keep_idempotency() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
        .execute(&pool).await.unwrap();
    let mut setup = db::begin_write(&pool).await.unwrap();
    crate::spaces::structure::initialize_empty_in_transaction(&mut setup, "space")
        .await
        .unwrap();
    setup.commit().await.unwrap();
    let now = crate::clock::unix_time_millis().unwrap();
    for kind in [PrincipalKind::BuiltinAgent, PrincipalKind::ExternalAgent] {
        let grant_id = kind.as_str();
        let mut tx = db::begin_write(&pool).await.unwrap();
        create_grant(
            &mut tx,
            &AuthorizationContext {
                principal: Principal {
                    kind,
                    id: grant_id.into(),
                },
                grant_id: grant_id.into(),
                home_space_id: "space".into(),
                read_space_ids: vec!["space".into()],
                write_space_ids: vec!["space".into()],
                action_ids: vec!["create-note".into()],
                valid_until: now + 60000,
            },
            now,
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();
        let request = ActionInvocationRequest {
            invocation_id: grant_id.into(),
            idempotency_key: grant_id.into(),
            grant_id: grant_id.into(),
            space_id: "space".into(),
            action_id: "create-note".into(),
            action_version: 1,
            parameters: json!({"title":"Result", "content":"Created autonomously"}),
            input_snapshot: Value::Null,
            expected_revisions: Value::Null,
            parent_run_id: None,
        };
        let outcome = invoke(&pool, request.clone()).await.unwrap();
        let ActionInvocationOutcome::CommittedReceipt { receipt } = outcome else {
            panic!("must commit directly")
        };
        let repeat = invoke(&pool, request).await.unwrap();
        let ActionInvocationOutcome::CommittedReceipt { receipt: repeated } = repeat else {
            panic!("must replay receipt")
        };
        assert_eq!(receipt, repeated);
        let mut observation = pool.begin().await.unwrap();
        let outputs = crate::spaces::conversations::outputs::collect(
            &mut observation,
            "space",
            &Principal {
                kind,
                id: grant_id.into(),
            },
        )
        .await
        .unwrap();
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].title, "Result");
        observation.commit().await.unwrap();
    }
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_nodes WHERE space_id='space'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 2);
    let nodes: Vec<String> =
        sqlx::query_scalar("SELECT data FROM spaces_nodes WHERE space_id='space'")
            .fetch_all(&pool)
            .await
            .unwrap();
    for encoded in nodes {
        let data = serde_json::from_str::<Value>(&encoded).unwrap();
        crate::spaces::knowledge::model::revision::validate_declared(&data).unwrap();
    }
}
