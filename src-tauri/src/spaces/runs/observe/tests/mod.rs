use super::*;

#[tokio::test]
async fn task_center_restores_roots_without_duplicate_child_actions() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
            .execute(&pool).await.unwrap();
    for (id, kind, parent, completed) in [
        ("workflow", "workflow", None, None),
        ("action", "action", None, None),
        ("child", "action", Some("workflow"), None),
        ("builtin_agent", "builtin_agent", None, None),
        ("completed", "action", None, Some(2_i64)),
    ] {
        sqlx::query("INSERT INTO spaces_runs(id,scope_kind,home_space_id,parent_run_id,principal_kind,principal_id,kind,state,policy_snapshot,input_snapshot,budget,created_at,updated_at,completed_at) VALUES($1,'space','space',$2,'user','local-user',$3,$4,'{}','{}','{}',1,1,$5)")
                .bind(id).bind(parent).bind(kind)
                .bind(if completed.is_some() { "succeeded" } else { "running" })
                .bind(completed).execute(&pool).await.unwrap();
    }
    let mut ids = active_resource_runs(&pool)
        .await
        .unwrap()
        .into_iter()
        .map(|run| run.id)
        .collect::<Vec<_>>();
    ids.sort();
    assert_eq!(ids, ["action", "workflow"]);
}
