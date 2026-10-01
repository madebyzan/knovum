use super::*;
use crate::spaces::db;

async fn fixture() -> sqlx::SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
            .execute(&pool)
            .await
            .unwrap();
    sqlx::query("INSERT INTO spaces_runs(id,home_space_id,principal_kind,principal_id,kind,state,policy_snapshot,input_snapshot,budget,created_at,updated_at) VALUES('run','space','external_agent','session','external_agent','running','{}','{}','{}',1,1)")
            .execute(&pool)
            .await
            .unwrap();
    pool
}

#[tokio::test]
async fn records_each_owned_path_once_and_releases_in_publication_transaction() {
    let pool = fixture().await;
    assert!(record_path(&pool, "run", "/managed/asset", PathRole::Asset)
        .await
        .unwrap());
    assert!(
        !record_path(&pool, "run", "/managed/asset", PathRole::Asset)
            .await
            .unwrap()
    );
    let mut tx = db::begin_write(&pool).await.unwrap();
    release(&mut tx, "run").await.unwrap();
    tx.commit().await.unwrap();
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_external_output_staging")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn run_deletion_cascades_its_manifest_only() {
    let pool = fixture().await;
    sqlx::query("INSERT INTO spaces_runs(id,home_space_id,principal_kind,principal_id,kind,state,policy_snapshot,input_snapshot,budget,created_at,updated_at) VALUES('other','space','external_agent','other','external_agent','running','{}','{}','{}',1,1)")
            .execute(&pool)
            .await
            .unwrap();
    record_path(&pool, "run", "/managed/one", PathRole::Asset)
        .await
        .unwrap();
    record_path(&pool, "other", "/managed/two", PathRole::Asset)
        .await
        .unwrap();
    sqlx::query("DELETE FROM spaces_runs WHERE id='run'")
        .execute(&pool)
        .await
        .unwrap();
    let paths: Vec<String> =
        sqlx::query_scalar("SELECT path FROM spaces_external_output_staging ORDER BY path")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(paths, vec!["/managed/two"]);
}

#[tokio::test]
async fn cancelled_run_cannot_claim_new_managed_paths() {
    let pool = fixture().await;
    sqlx::query("UPDATE spaces_runs SET cancel_requested_at=2 WHERE id='run'")
        .execute(&pool)
        .await
        .unwrap();
    assert!(record_path(&pool, "run", "/managed/late", PathRole::Asset)
        .await
        .is_err());
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_external_output_staging")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn reclamation_preserves_shared_claims_and_committed_references() {
    let pool = fixture().await;
    sqlx::query("INSERT INTO spaces_runs(id,home_space_id,principal_kind,principal_id,kind,state,policy_snapshot,input_snapshot,budget,created_at,updated_at) VALUES('other','space','external_agent','other','external_agent','running','{}','{}','{}',1,1)")
            .execute(&pool).await.unwrap();
    record_path(&pool, "other", "/managed/shared", PathRole::Embedded)
        .await
        .unwrap();
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES('node','space','image',0,0,$1,1,1)")
            .bind(r#"{"kind":"image","title":"Image","assetPath":"/managed/committed"}"#)
            .execute(&pool).await.unwrap();
    let paths = ["/managed/shared", "/managed/committed", "/managed/orphan"]
        .map(|path| (path.to_owned(), "embedded".into()));
    let mut tx = db::begin_write(&pool).await.unwrap();
    let removable = removable_paths(&mut tx, "run", &paths).await.unwrap();
    assert_eq!(
        removable,
        std::collections::BTreeSet::from(["/managed/orphan".into()])
    );
}
