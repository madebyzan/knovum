
#[tokio::test]
async fn nullable_json_stays_absent_and_malformed_json_is_rejected() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("database");
    let row = sqlx::query(
        "SELECT NULL AS absent, '{broken' AS damaged, '{\"title\":\"Research\"}' AS content",
    )
    .fetch_one(&pool)
    .await
    .expect("row");
    assert_eq!(
        super::decode_optional_json(&row, "absent").expect("optional"),
        None
    );
    assert_eq!(
        super::decode_json(&row, "content").expect("content")["title"],
        "Research"
    );
    assert!(super::decode_json(&row, "damaged").is_err());
    assert!(super::decode_optional_json(&row, "damaged").is_err());
}
