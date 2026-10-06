use serde_json::{json, Value};

#[tokio::test]
async fn frozen_execution_preserves_image_input_and_space_snapshot() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
        .execute(&pool).await.unwrap();
    let request = crate::spaces::conversations::StartBuiltinAgentTaskRequest {
        space_id: "space".into(),
        conversation_id: None,
        branch_id: None,
        prompt: "Describe this image".into(),
        mode: crate::spaces::conversations::ConversationMode::Execute,
        target: crate::ai::models::ModelTargetSelection::Model {
            connection_id: "connection".into(),
            model_id: "vision-model".into(),
            temperature: 0.0,
        },
        max_turns: 12,
        max_tool_calls: 32,
        max_total_tokens: 100_000,
        max_duration_seconds: 900,
        attachments: vec![],
        extensions: Default::default(),
    };
    let catalog = crate::spaces::conversations::tools::project(request.mode).unwrap();
    let ids = super::identity::TaskIds::new(&request);
    let grant = super::identity::grant(&request, &ids, &catalog, 1);
    use crate::agent_runtime::protocol::messages::{
        AgentContentPart, AgentInputMessage, AgentInputRole,
    };
    let messages = vec![AgentInputMessage {
        id: ids.input_id(),
        role: AgentInputRole::User,
        content: vec![
            AgentContentPart::Text {
                text: request.prompt.clone(),
            },
            AgentContentPart::Image {
                mime_type: "image/png".into(),
                data: "aW1hZ2U=".into(),
            },
        ],
    }];
    let target = crate::ai::models::ModelExecutionTarget {
        provider: "openai".into(),
        model: "frozen-vision-model".into(),
        base_url: "https://api.example/v1".into(),
        temperature: 0.0,
        secret_key: "credential-reference".into(),
        capabilities: vec![crate::ai::contracts::ExecutionCapability::VisionUnderstanding],
    };
    let profile = crate::ai::settings::contracts::BuiltinAgentModelSettings {
        protocol: crate::ai::contracts::ChatModelProtocol::OpenAiChat,
        context_window: 32768,
        max_output_tokens: 4096,
    };
    let mut tx = pool.begin().await.unwrap();
    let space =
        crate::spaces::queries::knowledge_projection::list_contents(&mut tx, "space", 0, 40)
            .await
            .unwrap();
    tx.commit().await.unwrap();
    let attachments = json!([{"kind":"image","nodeId":"image"}]);
    super::persist_task(
        &pool,
        super::NewBuiltinAgentTaskState {
            request: &request,
            ids: &ids,
            grant: &grant,
            catalog: &catalog,
            extensions: &serde_json::from_value(json!({"id":"snapshot","digest":"digest","skills":[],"files":[],"skillToolBindings":{},"connections":[],"accounts":[],"catalogs":{},"tools":[]})).unwrap(),
            context_manifest: &attachments,
            space_context: &space,
            attachment_summaries: &[],
            target: &target,
            profile: &profile,
            messages: &messages,
            history: &[],
            now: 1,
        },
    )
    .await
    .unwrap();
    let recovered = crate::spaces::conversations::repository::recoverable_task(&pool, &ids.run_id)
        .await
        .unwrap();
    assert!(
        matches!(&recovered.messages[0].content[1], AgentContentPart::Image { data, .. } if data == "aW1hZ2U=")
    );
    assert_eq!(recovered.frozen_target.model, "frozen-vision-model");
    assert_eq!(recovered.frozen_target.secret_key, "credential-reference");
    assert_eq!(
        recovered.profile_version,
        super::BUILTIN_AGENT_PROFILE_VERSION
    );
    let input: String = sqlx::query_scalar("SELECT input_snapshot FROM spaces_runs WHERE id=$1")
        .bind(&ids.run_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let input: Value = serde_json::from_str(&input).unwrap();
    assert_eq!(input["contextManifest"], attachments);
    assert_eq!(input["spaceContext"]["spaceId"], "space");
}
