use sqlx::sqlite::SqlitePoolOptions;

#[tokio::test]
async fn image_object_removal_is_a_valid_derivation_action() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("database");
    super::migrate(&pool).await.expect("baseline");
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
        .execute(&pool).await.expect("space");
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES('output','space','image',0,0,'{\"kind\":\"image\",\"title\":\"Output\"}',1,1)")
        .execute(&pool).await.expect("image output");
    sqlx::query("INSERT INTO spaces_knowledge_derivations(id,space_id,output_node_id,action_kind,action_version,engine,status,created_at,updated_at) VALUES('job','space','output','remove-image-object',1,'dashscope:wanx2.1-imageedit:mask','running',1,1)")
        .execute(&pool).await.expect("object-removal derivation");
}

#[tokio::test]
async fn obsolete_migration_history_is_rejected_without_losing_user_rows() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("database");
    super::migrate(&pool).await.expect("baseline");
    sqlx::query(
        "INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('keep','space','Keep',1,1)",
    )
    .execute(&pool)
    .await
    .expect("user row");
    sqlx::query("INSERT INTO _sqlx_migrations(version,description,success,checksum,execution_time) VALUES(2,'obsolete',1,x'00',0)").execute(&pool).await.expect("old version");
    let error = super::migrate(&pool)
        .await
        .expect_err("obsolete history must fail");
    assert!(matches!(
        error,
        sqlx::migrate::MigrateError::VersionMissing(2)
    ));
    let title: String = sqlx::query_scalar("SELECT title FROM spaces WHERE id='keep'")
        .fetch_one(&pool)
        .await
        .expect("original data preserved");
    assert_eq!(title, "Keep");
}

#[tokio::test]
async fn obsolete_baseline_checksum_is_rejected_without_resetting_the_database() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("database");
    super::migrate(&pool).await.expect("baseline");
    sqlx::query(
        "INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('keep','space','Keep',1,1)",
    )
    .execute(&pool)
    .await
    .expect("user row");
    sqlx::query("UPDATE _sqlx_migrations SET checksum=x'00' WHERE version=1")
        .execute(&pool)
        .await
        .expect("obsolete baseline");
    assert!(matches!(
        super::migrate(&pool).await,
        Err(sqlx::migrate::MigrateError::VersionMismatch(1))
    ));
    let title: String = sqlx::query_scalar("SELECT title FROM spaces WHERE id='keep'")
        .fetch_one(&pool)
        .await
        .expect("original data preserved");
    assert_eq!(title, "Keep");
}
