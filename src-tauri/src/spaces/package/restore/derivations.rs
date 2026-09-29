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
            sqlx::query("INSERT INTO spaces_grounded_documents(derivation_id,citations,provider,model) VALUES($1,$2,$3,$4)")
                .bind(&row.id)
                .bind(serde_json::to_string(&document.citations).map_err(db::json_error)?)
                .bind(&document.provider).bind(&document.model).execute(&mut **tx).await.map_err(db::sql_error)?;
        }
        if let Some(image) = &row.generated_image {
            sqlx::query("INSERT INTO spaces_generated_images(derivation_id,intent,style,aspect_ratio,provider,model,revised_prompt) VALUES($1,$2,$3,$4,$5,$6,$7)")
                .bind(&row.id).bind(&image.intent).bind(&image.style).bind(&image.aspect_ratio)
                .bind(&image.provider).bind(&image.model).bind(&image.revised_prompt)
                .execute(&mut **tx).await.map_err(db::sql_error)?;
        }
        if let Some(video) = &row.generated_video {
            sqlx::query("INSERT INTO spaces_generated_videos(derivation_id,prompt,aspect_ratio,duration_seconds,provider,model,operation_id) VALUES($1,$2,$3,$4,$5,$6,$7)")
                .bind(&row.id).bind(&video.prompt).bind(&video.aspect_ratio).bind(video.duration_seconds)
                .bind(&video.provider).bind(&video.model).bind(&video.operation_id)
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
            metadata.citations.is_array()
                && !metadata.provider.trim().is_empty()
                && !metadata.model.trim().is_empty()
        }
        ("draft-document", None) => row.status == "failed",
        (_, None) => true,
        (_, Some(_)) => false,
    };
    let image = match (&row.action_kind[..], &row.generated_image) {
        ("generate-visualization", Some(metadata)) => {
            !metadata.intent.trim().is_empty()
                && !metadata.aspect_ratio.trim().is_empty()
                && !metadata.provider.trim().is_empty()
                && !metadata.model.trim().is_empty()
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
                && !metadata.provider.trim().is_empty()
                && !metadata.model.trim().is_empty()
                && !metadata.operation_id.trim().is_empty()
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
        serde_json::json!({"id":"derivation","outputNodeId":"video","inputs":[{"nodeId":"source","sourceIdentity":"source","sourceSnapshot":{"title":"Source","kind":"markdown","originalUrl":null,"platform":null},"role":"primary","sourceRevision":"node1:source","revisionBasis":"knowledge","inputRevision":"node1:source","inputSnapshot":null}],"actionKind":"generate-video","actionVersion":1,"engine":"qwen:wan3.0-video","status":"completed","error":"","createdAt":1,"updatedAt":2,"completedAt":2,"generatedVideo":{"prompt":"Explain the source","aspectRatio":"16:9","durationSeconds":10,"provider":"qwen","model":"wan3.0-video","operationId":"task-1"}})
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
}
