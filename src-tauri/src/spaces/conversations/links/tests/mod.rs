use serde_json::json;

use super::*;

fn workspace() -> PathBuf {
    PathBuf::from("/Users/zand/agent-workspaces/spaces/space/conversation")
}

#[test]
fn a_destination_names_the_file_its_writer_saw() {
    // Rendered Markdown percent-encodes the destination the Agent wrote in angle brackets.
    let encoded = "/Users/zand/agent-workspaces/spaces/space/conversation/PPT%E6%91%98%E8%A6%81.md";
    assert_eq!(
        destination(encoded, &workspace()),
        Some(workspace().join("PPT摘要.md"))
    );
    assert_eq!(
        destination("notes/./PPT%20summary.md#L3", &workspace()),
        Some(workspace().join("notes/PPT summary.md"))
    );
    assert_eq!(
        destination("../conversation/report.md?plain", &workspace()),
        Some(workspace().join("report.md"))
    );
}

#[test]
fn a_destination_that_names_no_file_is_refused() {
    assert_eq!(destination("", &workspace()), None);
    assert_eq!(destination("#section", &workspace()), None);
    assert_eq!(destination("%FF.md", &workspace()), None);
    assert_eq!(destination("/../../etc/passwd", &workspace()), None);
}

async fn fixture() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    db::migrate(&pool).await.unwrap();
    for statement in [
        "INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)",
        "INSERT INTO spaces_conversations(id,space_id,title,created_at,updated_at,active_branch_id)
         VALUES('conversation','space','Agent',1,1,'main')",
    ] {
        sqlx::query(statement).execute(&pool).await.unwrap();
    }
    pool
}

async fn node(pool: &SqlitePool, id: &str, deleted: bool) {
    insert(
        pool,
        id,
        "markdown",
        json!({"kind":"markdown","title":"PPT摘要","text":""}),
        deleted,
    )
    .await;
}

async fn insert(pool: &SqlitePool, id: &str, kind: &str, data: serde_json::Value, deleted: bool) {
    sqlx::query(
        "INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at,deleted_at)
         VALUES($1,'space',$2,0,0,$3,1,1,$4)",
    )
    .bind(id)
    .bind(kind)
    .bind(data.to_string())
    .bind(deleted.then_some(2))
    .execute(pool)
    .await
    .unwrap();
}

/// A message on `branch` at `created_at`, publishing `outputs` as (node id, workspace path).
async fn message(
    pool: &SqlitePool,
    id: &str,
    branch: &str,
    created_at: i64,
    outputs: &[(&str, &str)],
) {
    let outputs: Vec<_> = outputs
        .iter()
        .map(|(node_id, path)| {
            json!({"nodeId": node_id, "title": "PPT摘要", "kind": "markdown", "workspacePath": path})
        })
        .collect();
    let content = json!({"parts": [], "outputs": outputs, "citations": []});
    sqlx::query(
        "INSERT INTO spaces_conversation_messages(id,conversation_id,branch_id,role,content,created_at)
         VALUES($1,'conversation',$2,'assistant',$3,$4)",
    )
    .bind(id)
    .bind(branch)
    .bind(content.to_string())
    .bind(created_at)
    .execute(pool)
    .await
    .unwrap();
}

async fn follow_from(pool: &SqlitePool, message_id: &str, relative: &str) -> Option<Deliverable> {
    let message = linking_message(pool, message_id).await.unwrap();
    match deliverable(pool, &message, Path::new(relative)).await {
        Ok(deliverable) => Some(deliverable),
        Err(error) => {
            assert_eq!(error.code, "conversation_link_unavailable");
            None
        }
    }
}

fn document(node_id: &str) -> Option<Deliverable> {
    Some(Deliverable::Document(node_id.into()))
}

#[tokio::test]
async fn a_deliverable_link_follows_to_the_node_its_turn_published() {
    let pool = fixture().await;
    node(&pool, "first", false).await;
    node(&pool, "second", false).await;
    // The answer that links it, then the turn's final message carrying what it published.
    message(&pool, "commentary", "main", 10, &[]).await;
    message(&pool, "answer", "main", 11, &[("first", "PPT摘要.md")]).await;
    // A later turn wrote the same file name into the emptied directory again.
    message(&pool, "later", "main", 20, &[("second", "PPT摘要.md")]).await;

    assert_eq!(
        follow_from(&pool, "commentary", "PPT摘要.md").await,
        document("first")
    );
    assert_eq!(
        follow_from(&pool, "answer", "PPT摘要.md").await,
        document("first")
    );
    assert_eq!(
        follow_from(&pool, "later", "PPT摘要.md").await,
        document("second")
    );
}

#[tokio::test]
async fn a_deliverable_that_became_a_file_opens_the_file_the_node_holds() {
    let pool = fixture().await;
    let asset = "/Users/zand/Library/Application Support/knovum/spaces/assets/import-1/deck.pptx";
    insert(
        &pool,
        "deck",
        "file",
        json!({"kind":"file","title":"deck.pptx","fileName":"deck.pptx","assetPath":asset}),
        false,
    )
    .await;
    message(&pool, "answer", "main", 10, &[("deck", "out/deck.pptx")]).await;

    assert_eq!(
        follow_from(&pool, "answer", "out/deck.pptx").await,
        Some(Deliverable::File(PathBuf::from(asset)))
    );
}

#[tokio::test]
async fn a_deliverable_link_never_follows_to_another_branch_or_a_deleted_node() {
    let pool = fixture().await;
    node(&pool, "elsewhere", false).await;
    node(&pool, "deleted", true).await;
    message(&pool, "answer", "main", 10, &[("deleted", "PPT摘要.md")]).await;
    message(&pool, "branch", "other", 11, &[("elsewhere", "PPT摘要.md")]).await;

    assert_eq!(follow_from(&pool, "answer", "PPT摘要.md").await, None);
    assert_eq!(follow_from(&pool, "answer", "other.md").await, None);
}
