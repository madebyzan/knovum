use sqlx::{Sqlite, Transaction};

use super::contracts::{PackageDerivation, RestorePackageRequest};
use crate::spaces::{
    db,
    derivation::input_revision::{self, RevisionBasis},
};

pub(super) async fn insert(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
) -> Result<(), String> {
    for row in &request.derivations {
        validate(row)?;
        sqlx::query(
            "INSERT INTO spaces_knowledge_derivations(id,space_id,output_node_id,action_kind,action_version,engine,status,error,created_at,updated_at,completed_at) \
             VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)",
        )
        .bind(&row.id).bind(&request.space.id).bind(&row.output_node_id)
        .bind(&row.action_kind).bind(row.action_version).bind(&row.engine)
        .bind(&row.status).bind(&row.error).bind(row.created_at).bind(row.updated_at).bind(row.completed_at)
        .execute(&mut **tx).await.map_err(db::sql_error)?;
        for (position, input) in row.inputs.iter().enumerate() {
            if input.source_identity.trim().is_empty()
                || input.source_revision.trim().is_empty()
                || input.source_snapshot.kind.trim().is_empty()
                || !matches!(
                    input.role.as_str(),
                    "primary" | "source" | "context" | "style-reference"
                )
            {
                return Err("Package derivation source is invalid".into());
            }
            let basis = RevisionBasis::parse(&input.revision_basis)?;
            if input_revision::from_snapshot(
                &input.source_snapshot.kind,
                &input.source_revision,
                basis,
                &input.input_snapshot,
            )? != input.input_revision
            {
                return Err("Package derivation input revision is invalid".into());
            }
            sqlx::query("INSERT INTO spaces_knowledge_derivation_inputs(derivation_id,position,node_id,role,source_revision,revision_basis,input_revision,input_snapshot,source_identity,source_snapshot) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)")
                .bind(&row.id).bind(position as i64).bind(&input.node_id).bind(&input.role)
                .bind(&input.source_revision).bind(&input.revision_basis).bind(&input.input_revision)
                .bind(serde_json::to_string(&input.input_snapshot).map_err(db::json_error)?)
                .bind(&input.source_identity)
                .bind(serde_json::to_string(&input.source_snapshot).map_err(db::json_error)?)
                .execute(&mut **tx).await.map_err(db::sql_error)?;
        }
        if let Some(document) = &row.grounded_document {
            sqlx::query("INSERT INTO spaces_grounded_documents(derivation_id,citations,execution_source) VALUES($1,$2,$3)")
                .bind(&row.id)
                .bind(serde_json::to_string(&document.citations).map_err(db::json_error)?)
                .bind(serde_json::to_string(&document.execution_source).map_err(db::json_error)?).execute(&mut **tx).await.map_err(db::sql_error)?;
        }
        if let Some(image) = &row.generated_image {
            sqlx::query("INSERT INTO spaces_generated_images(derivation_id,intent,style,aspect_ratio,execution_source,revised_prompt) VALUES($1,$2,$3,$4,$5,$6)")
                .bind(&row.id).bind(&image.intent).bind(&image.style).bind(&image.aspect_ratio)
                .bind(serde_json::to_string(&image.execution_source).map_err(db::json_error)?).bind(&image.revised_prompt)
                .execute(&mut **tx).await.map_err(db::sql_error)?;
        }
        if let Some(video) = &row.generated_video {
            sqlx::query("INSERT INTO spaces_generated_videos(derivation_id,prompt,aspect_ratio,duration_seconds,execution_source,operation_id) VALUES($1,$2,$3,$4,$5,$6)")
                .bind(&row.id).bind(&video.prompt).bind(&video.aspect_ratio).bind(video.duration_seconds)
                .bind(serde_json::to_string(&video.execution_source).map_err(db::json_error)?).bind(&video.operation_id)
                .execute(&mut **tx).await.map_err(db::sql_error)?;
        }
    }
    Ok(())
}

fn validate(row: &PackageDerivation) -> Result<(), String> {
    if let Some(document) = &row.grounded_document {
        crate::spaces::knowledge::grounding::validate_citation_snapshots(&document.citations)?;
    }
    let action = matches!(
        row.action_kind.as_str(),
        "remove-image-background"
            | "remove-image-object"
            | "split-image-layers"
            | "upscale-image"
            | "generate-image-angle"
            | "generate-visualization"
            | "extract-subtitles"
            | "render-subtitled-video"
            | "render-video-edit"
            | "draft-document"
            | "export-document-as-docx"
            | "export-document-as-pdf"
            | "export-document-as-txt"
            | "render-presentation"
            | "generate-video"
    ) && row.action_version == 1;
    let state = (row.status == "completed" && row.error.is_empty())
        || (row.status == "failed" && !row.error.trim().is_empty());
    let primary = row
        .inputs
        .iter()
        .filter(|input| input.role == "primary")
        .count();
    let inputs = primary == 1
        && row
            .inputs
            .first()
            .is_some_and(|input| input.role == "primary");
    let document = match (&row.action_kind[..], &row.grounded_document) {
        ("draft-document", Some(metadata)) => {
            metadata.citations.is_array() && metadata.execution_source.validate().is_ok()
        }
        ("draft-document", None) => row.status == "failed",
        (_, None) => true,
        (_, Some(_)) => false,
    };
    let image = match (&row.action_kind[..], &row.generated_image) {
        ("generate-visualization", Some(metadata)) => {
            !metadata.intent.trim().is_empty()
                && !metadata.aspect_ratio.trim().is_empty()
                && metadata.execution_source.validate().is_ok()
        }
        ("generate-visualization", None) => row.status == "failed",
        (_, None) => true,
        (_, Some(_)) => false,
    };
    let video = match (&row.action_kind[..], &row.generated_video) {
        ("generate-video", Some(metadata)) => {
            !metadata.prompt.trim().is_empty()
                && matches!(metadata.aspect_ratio.as_str(), "1:1" | "16:9" | "9:16")
                && (5..=15).contains(&metadata.duration_seconds)
                && metadata.execution_source.validate().is_ok()
                && match &metadata.execution_source {
                    crate::ai::tasks::ExecutionSource::Model { .. } => metadata
                        .operation_id
                        .as_ref()
                        .is_some_and(|id| !id.trim().is_empty()),
                    crate::ai::tasks::ExecutionSource::Agent { .. } => {
                        metadata.operation_id.is_none()
                    }
                }
        }
        ("generate-video", None) => row.status == "failed",
        (_, None) => true,
        (_, Some(_)) => false,
    };
    if action && state && inputs && document && image && video {
        Ok(())
    } else {
        Err("Package knowledge derivation is invalid".into())
    }
}

#[cfg(test)]
mod tests {
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
}
