use serde_json::{json, Value};

#[tokio::test]
async fn space_snapshot_preserves_media_based_vision_recovery() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
        .execute(&pool).await.unwrap();
    let request = crate::spaces::assistant::StartAssistantTaskRequest {
        space_id: "space".into(),
        conversation_id: None,
        branch_id: None,
        prompt: "Describe this image".into(),
        mode: crate::spaces::assistant::AssistantMode::Execute,
        target: crate::ai::models::ExecutionTargetSelection::Model {
            connection_id: "connection".into(),
            model_id: "vision-model".into(),
            temperature: 0.0,
        },
        max_turns: 12,
        max_tool_calls: 32,
        max_total_tokens: 100_000,
        max_duration_seconds: 900,
        attachments: vec![],
    };
    let catalog = crate::spaces::assistant::tools::project(request.mode).unwrap();
    let ids = super::identity::TaskIds::new(&request);
    let grant = super::identity::grant(&request, &ids, &catalog, 1);
    let checkpoint = crate::agent_runtime::start(
        crate::ai::contracts::ModelMessage {
            role: crate::ai::contracts::ModelRole::User,
            content: vec![
                crate::ai::contracts::ModelContentPart::Text {
                    text: request.prompt.clone(),
                },
                crate::ai::contracts::ModelContentPart::Binary {
                    media_type: "image/png".into(),
                    source: crate::ai::contracts::ModelBinarySource::Base64 {
                        data: "aW1hZ2U=".into(),
                    },
                    name: Some("image.png".into()),
                },
            ],
        },
        request.max_turns,
        request.max_tool_calls,
        ids.run_id.clone(),
        "segment".into(),
        catalog.digest.clone(),
    )
    .unwrap();
    let mut tx = pool.begin().await.unwrap();
    let space =
        crate::spaces::queries::knowledge_projection::list_contents(&mut tx, "space", 0, 40)
            .await
            .unwrap();
    tx.commit().await.unwrap();
    let attachments = json!([{"kind":"image","nodeId":"image"}]);
    super::persist_task(
        &pool,
        super::NewAssistantTaskState {
            request: &request,
            ids: &ids,
            grant: &grant,
            catalog: &catalog,
            context_manifest: &attachments,
            space_context: &space,
            attachment_summaries: &[],
            checkpoint: &checkpoint,
            now: 1,
        },
    )
    .await
    .unwrap();
    let recovered = crate::spaces::assistant::repository::recoverable_task(&pool, &ids.run_id)
        .await
        .unwrap();
    assert!(recovered.requires_vision);
    assert_eq!(recovered.profile_version, super::ASSISTANT_PROFILE_VERSION);
    let input: String = sqlx::query_scalar("SELECT input_snapshot FROM spaces_runs WHERE id=$1")
        .bind(&ids.run_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let input: Value = serde_json::from_str(&input).unwrap();
    assert_eq!(input["contextManifest"], attachments);
    assert_eq!(input["spaceContext"]["spaceId"], "space");
}
