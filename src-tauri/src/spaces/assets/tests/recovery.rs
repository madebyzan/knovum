use sqlx::sqlite::SqlitePoolOptions;
use sqlx::Row;

#[tokio::test]
async fn recovery_preserves_unstarted_run_outputs_and_catches_interrupted_work() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
        .execute(&pool)
        .await
        .unwrap();
    for id in [
        "active-output",
        "started-output",
        "finished-output",
        "orphan",
    ] {
        sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES($1,'space','image',0,0,'{\"kind\":\"image\",\"title\":\"Pending\",\"processingState\":\"processing\"}',1,1)")
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
    }
    for (id, state) in [
        ("active", "queued"),
        ("started", "running"),
        ("finished", "failed"),
    ] {
        sqlx::query("INSERT INTO spaces_runs(id,scope_kind,home_space_id,principal_kind,principal_id,kind,state,policy_snapshot,input_snapshot,budget,created_at,updated_at) VALUES($1,'space','space','user','user','action',$2,'{}','{}','{}',1,1)")
            .bind(id)
            .bind(state)
            .execute(&pool)
            .await
            .unwrap();
    }
    sqlx::query("INSERT INTO spaces_knowledge_derivations(id,space_id,output_node_id,action_kind,action_version,engine,status,created_at,updated_at) VALUES('started','space','started-output','generate-image',1,'test','running',1,1)")
        .execute(&pool)
        .await
        .unwrap();
    let rows = sqlx::query(super::INTERRUPTED_ASSETS)
        .fetch_all(&pool)
        .await
        .unwrap();
    let mut ids = rows
        .iter()
        .map(|row| row.try_get::<String, _>("id").unwrap())
        .collect::<Vec<_>>();
    ids.sort();
    assert_eq!(ids, ["finished-output", "orphan", "started-output"]);
}
