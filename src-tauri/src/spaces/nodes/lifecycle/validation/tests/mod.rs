use sqlx::sqlite::SqlitePoolOptions;

#[tokio::test]
async fn trash_accepts_section_node() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("sqlite");
    crate::spaces::db::migrate(&pool)
        .await
        .expect("spaces migrations");
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at,last_opened_at) VALUES ('space','space','Space',1,1,1)")
        .execute(&pool)
        .await
        .expect("space");
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES ('section_1','space','section',0,0,'{\"kind\":\"section\",\"title\":\"Group 1\",\"width\":400,\"height\":300}',1,1)")
        .execute(&pool)
        .await
        .expect("section");

    let mut tx = pool.begin().await.expect("begin");
    let validated = super::active_unique_ids(&mut tx, "space", &["section_1".to_string()], true)
        .await
        .expect("section node should be valid for trash");
    assert_eq!(validated, vec!["section_1".to_string()]);
    tx.rollback().await.expect("rollback");
}
