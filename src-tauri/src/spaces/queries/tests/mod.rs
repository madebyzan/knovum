use sqlx::sqlite::SqlitePoolOptions;

#[tokio::test]
async fn damaged_coordinate_is_reported_instead_of_panicking_or_defaulting() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("database");
    crate::spaces::db::migrate(&pool).await.expect("schema");
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
        .execute(&pool).await.expect("space");
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES('node','space','image',0,0,$1,1,1)")
        .bind(r#"{"kind":"image","title":"Image"}"#)
        .execute(&pool).await.expect("node");
    sqlx::query("UPDATE spaces_nodes SET initial_x='damaged-coordinate' WHERE id='node'")
        .execute(&pool)
        .await
        .expect("simulate corrupted SQLite value");
    let mut tx = pool.begin().await.expect("read snapshot");
    let result = super::nodes::list_in_transaction(&mut tx, "space").await;
    let error = result.err().expect("damaged value must fail explicitly");
    assert!(error.contains("initial_x"), "{error}");
}
