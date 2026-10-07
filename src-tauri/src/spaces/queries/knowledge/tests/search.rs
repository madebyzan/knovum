use serde_json::json;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

async fn database() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    for space in ["one", "two"] {
        sqlx::query(
            "INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES($1,'space',$1,1,1)",
        )
        .bind(space)
        .execute(&pool)
        .await
        .unwrap();
    }
    for (id, space, title) in [
        ("alpha", "one", "bob's alpha"),
        ("beta", "one", "bob's beta"),
        ("other", "two", "bob's alpha"),
        ("deleted", "one", "bob's alpha"),
    ] {
        let mut data = json!({"kind":"image","title":title,"sourceRevision":"asset-r1","assetPath":"/managed/image.png"});
        crate::spaces::knowledge::indexing::revision::apply(&mut data).unwrap();
        sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES($1,$2,'image',0,0,$3,1,1)")
            .bind(id).bind(space).bind(data.to_string()).execute(&pool).await.unwrap();
    }
    sqlx::query("UPDATE spaces_nodes SET deleted_at=2 WHERE id='deleted'")
        .execute(&pool)
        .await
        .unwrap();
    pool
}

#[tokio::test]
async fn substring_search_binds_quotes_terms_and_node_filters() {
    let pool = database().await;
    let rows = super::search_space(
        &pool,
        "one".into(),
        None,
        vec!["bob's".into(), "alpha".into()],
        Some(vec![
            "alpha".into(),
            "beta".into(),
            "other".into(),
            "deleted".into(),
        ]),
    )
    .await
    .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["nodeId"], "alpha");
    for (terms, ids) in [
        (vec!["' OR 1=1 --".into()], None),
        (vec!["alpha".into()], Some(Vec::new())),
        (vec!["alpha".into()], Some(vec!["beta".into()])),
    ] {
        assert!(super::search_space(&pool, "one".into(), None, terms, ids)
            .await
            .unwrap()
            .is_empty());
    }
    let rows = super::search_global(&pool, None, vec!["bob's".into(), "alpha".into()])
        .await
        .unwrap();
    let ids = rows
        .iter()
        .map(|row| row["nodeId"].as_str().unwrap())
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(ids, std::collections::HashSet::from(["alpha", "other"]));
}

#[tokio::test]
async fn fts_search_keeps_node_filter_bindings_after_the_match_argument() {
    let pool = database().await;
    let plan = crate::spaces::knowledge::search_plan::build("alpha").unwrap();
    assert!(plan.fts_query.is_some());
    let rows = super::search_space(
        &pool,
        "one".into(),
        plan.fts_query,
        plan.terms,
        Some(vec!["beta".into()]),
    )
    .await
    .unwrap();
    assert!(rows.is_empty());
    let plan = crate::spaces::knowledge::search_plan::build("alpha").unwrap();
    let rows = super::search_space(
        &pool,
        "one".into(),
        plan.fts_query,
        plan.terms,
        Some(vec!["alpha".into()]),
    )
    .await
    .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["nodeId"], "alpha");
}
