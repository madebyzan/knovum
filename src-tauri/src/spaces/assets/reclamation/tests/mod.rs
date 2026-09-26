use sqlx::sqlite::SqlitePoolOptions;
use std::collections::BTreeSet;

#[tokio::test]
async fn generated_preview_stays_protected_until_its_last_durable_reference_is_removed() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("database");
    crate::spaces::db::migrate(&pool).await.expect("schema");
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
        .execute(&pool).await.expect("space");
    for id in ["first", "second"] {
        sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES($1,'space','image',0,0,$2,1,1)")
            .bind(id).bind(r#"{"kind":"image","title":"Image","previewPath":"/previews/shared.png"}"#)
            .execute(&pool).await.expect("node");
    }
    let candidates = BTreeSet::from(["/previews/shared.png".into(), "/previews/orphan.png".into()]);
    assert_eq!(
        super::unreferenced_paths(&pool, candidates.clone())
            .await
            .expect("references"),
        ["/previews/orphan.png"]
    );
    sqlx::query("UPDATE spaces_nodes SET deleted_at=2 WHERE id='first'")
        .execute(&pool)
        .await
        .expect("soft delete");
    assert_eq!(
        super::unreferenced_paths(&pool, candidates.clone())
            .await
            .expect("retained references"),
        ["/previews/orphan.png"]
    );
    sqlx::query("DELETE FROM spaces_nodes")
        .execute(&pool)
        .await
        .expect("permanent delete");
    assert_eq!(
        super::unreferenced_paths(&pool, candidates)
            .await
            .expect("no references")
            .len(),
        2
    );
}
