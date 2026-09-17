use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

pub(super) async fn database() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("sqlite");
    crate::spaces::db::migrate(&pool)
        .await
        .expect("spaces migrations");
    pool
}

pub(super) async fn seed_relation(pool: &SqlitePool, status: &str) {
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at,last_opened_at) VALUES ('space','space','Space',1,1,1)")
        .execute(pool).await.expect("space");
    for id in ["source", "target"] {
        let data = format!(r#"{{"kind":"note","title":"{id}","body":"","tags":[]}}"#);
        sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES ($1,'space','note',0,0,$2,1,1)")
            .bind(id).bind(data).execute(pool).await.expect("node");
    }
    sqlx::query("INSERT INTO spaces_relations(id,space_id,source_id,target_id,kind,rationale,status,actor_kind,created_at,updated_at) VALUES ('relation','space','source','target','supports','',$1,'human',1,1)")
        .bind(status).execute(pool).await.expect("relation");
}
