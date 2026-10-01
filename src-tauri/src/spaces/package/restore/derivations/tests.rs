
use super::*;
fn derivation() -> serde_json::Value {
    serde_json::json!({"id":"derivation","outputNodeId":"video","inputs":[{"nodeId":"source","sourceIdentity":"source","sourceSnapshot":{"title":"Source","kind":"markdown","originalUrl":null,"platform":null},"role":"primary","sourceRevision":"node1:source","revisionBasis":"knowledge","inputRevision":"node1:source","inputSnapshot":null}],"actionKind":"generate-video","actionVersion":1,"engine":"qwen:wan3.0-video","status":"completed","error":"","createdAt":1,"updatedAt":2,"completedAt":2,"generatedVideo":{"prompt":"Explain the source","aspectRatio":"16:9","durationSeconds":10,"executionSource":{"type":"model","provider":"qwen","model":"wan3.0-video"},"operationId":"task-1"}})
}
#[test]
fn requires_current_metadata_for_completed_video() {
    let mut value = derivation();
    assert!(validate(&serde_json::from_value(value.clone()).unwrap()).is_ok());
    value.as_object_mut().unwrap().remove("generatedVideo");
    assert!(validate(&serde_json::from_value(value).unwrap()).is_err());
    let mut obsolete = derivation();
    obsolete["generatedVideo"]["reviewState"] = serde_json::json!("reviewed");
    assert!(serde_json::from_value::<PackageDerivation>(obsolete).is_err());
}
#[test]
fn agent_video_records_runtime_without_a_fake_provider_operation() {
    let mut value = derivation();
    value["generatedVideo"]["executionSource"] =
        serde_json::json!({"type":"agent","runtimeKind":"codex"});
    value["generatedVideo"]["operationId"] = serde_json::Value::Null;
    assert!(validate(&serde_json::from_value(value.clone()).unwrap()).is_ok());
    value["generatedVideo"]["operationId"] = serde_json::json!("fake-operation");
    assert!(validate(&serde_json::from_value(value.clone()).unwrap()).is_err());
    value["generatedVideo"]["provider"] = serde_json::json!("codex");
    assert!(serde_json::from_value::<PackageDerivation>(value).is_err());
}
#[test]
fn accepts_document_export_without_provider_metadata() {
    let mut value = derivation();
    value.as_object_mut().unwrap().remove("generatedVideo");
    value["actionKind"] = serde_json::json!("export-document-as-pdf");
    value["engine"] = serde_json::json!("typst:0.15.1");
    assert!(validate(&serde_json::from_value(value).unwrap()).is_ok());
}
#[test]
fn accepts_object_removal_without_generation_metadata() {
    let mut value = derivation();
    value.as_object_mut().unwrap().remove("generatedVideo");
    value["actionKind"] = serde_json::json!("remove-image-object");
    value["engine"] = serde_json::json!("dashscope:wanx2.1-imageedit:mask");
    value["outputNodeId"] = serde_json::json!("image-output");
    value["inputs"][0]["sourceSnapshot"]["kind"] = serde_json::json!("image");
    value["inputs"][0]["revisionBasis"] = serde_json::json!("rendered");
    let snapshot = serde_json::json!({ "imageEdit": null });
    value["inputs"][0]["inputRevision"] = serde_json::json!(input_revision::from_snapshot(
        "image",
        "node1:source",
        RevisionBasis::Rendered,
        &snapshot,
    )
    .unwrap());
    value["inputs"][0]["inputSnapshot"] = snapshot;
    assert!(validate(&serde_json::from_value(value).unwrap()).is_ok());
}
#[test]
fn generated_metadata_matches_the_actual_action_identity() {
    for (actions, field, metadata) in [
        (
            vec![
                "draft-document",
                "draft-script",
                "summarize-meeting",
                "draft-social-post",
            ],
            "groundedDocument",
            serde_json::json!({"citations":[],"executionSource":{"type":"agent","runtimeKind":"codex"}}),
        ),
        (
            vec!["generate-visualization", "generate-image"],
            "generatedImage",
            serde_json::json!({"intent":"Explain","style":"Flat","aspectRatio":"1:1","executionSource":{"type":"agent","runtimeKind":"codex"},"revisedPrompt":null}),
        ),
    ] {
        for action in actions {
            let mut value = derivation();
            value.as_object_mut().unwrap().remove("generatedVideo");
            value["actionKind"] = serde_json::json!(action);
            value[field] = metadata.clone();
            assert!(
                validate(&serde_json::from_value(value).unwrap()).is_ok(),
                "{action}"
            );
        }
    }
}
#[tokio::test]
async fn restores_agent_execution_sources_into_the_current_sql_baseline() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)").execute(&pool).await.unwrap();
    for id in ["source", "document", "image", "video"] {
        sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES($1,'space','markdown',0,0,$2,1,1)")
                .bind(id).bind(serde_json::json!({"kind":"markdown","title":"Record","text":"Body"}).to_string()).execute(&pool).await.unwrap();
    }
    let source = serde_json::json!({"type":"agent","runtimeKind":"codex"});
    let mut rows = Vec::new();
    for (id, action, field, metadata) in [
        (
            "document",
            "draft-document",
            "groundedDocument",
            serde_json::json!({"citations":[],"executionSource":source}),
        ),
        (
            "image",
            "generate-visualization",
            "generatedImage",
            serde_json::json!({"intent":"Explain","style":"","aspectRatio":"1:1","executionSource":source,"revisedPrompt":null}),
        ),
        (
            "video",
            "generate-video",
            "generatedVideo",
            serde_json::json!({"prompt":"Explain","aspectRatio":"16:9","durationSeconds":10,"executionSource":source,"operationId":null}),
        ),
    ] {
        let mut row = derivation();
        row.as_object_mut().unwrap().remove("generatedVideo");
        row["id"] = serde_json::json!(id);
        row["outputNodeId"] = serde_json::json!(id);
        row["actionKind"] = serde_json::json!(action);
        row["engine"] = serde_json::json!("desktop-agent:codex");
        row[field] = metadata;
        rows.push(row);
    }
    let request: RestorePackageRequest = serde_json::from_value(serde_json::json!({
            "space":{"id":"space","kind":"space","title":"Space","lastOpenedAt":1,"createdAt":1,"updatedAt":1},
            "structureNodes":[],"layouts":[],"relationStyles":[],"objectTypes":[],"nodes":[],"scenes":[],"relations":[],"derivations":rows,
            "runs":[],"runSteps":[],"runAttempts":[],"runEvents":[],"externalAgentConversations":[],"externalAgentTurns":[],"assistantConversations":[],"assistantMessages":[],"provenance":[],"knowledgeEvents":[],"chunks":[]
        })).unwrap();
    let mut tx = pool.begin().await.unwrap();
    insert(&mut tx, &request).await.unwrap();
    tx.commit().await.unwrap();
    let sources: Vec<String> = sqlx::query_scalar("SELECT execution_source FROM spaces_grounded_documents UNION ALL SELECT execution_source FROM spaces_generated_images UNION ALL SELECT execution_source FROM spaces_generated_videos").fetch_all(&pool).await.unwrap();
    assert_eq!(sources.len(), 3);
    for raw in sources {
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&raw).unwrap(),
            source
        );
    }
    let operation: Option<String> =
        sqlx::query_scalar("SELECT operation_id FROM spaces_generated_videos")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(operation.is_none());
}
