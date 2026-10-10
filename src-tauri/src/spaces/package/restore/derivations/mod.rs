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
                .bind(sqlx::types::Json(&input.input_snapshot))
                .bind(&input.source_identity)
                .bind(sqlx::types::Json(&input.source_snapshot))
                .execute(&mut **tx).await.map_err(db::sql_error)?;
        }
        if let Some(document) = &row.grounded_document {
            sqlx::query("INSERT INTO spaces_grounded_documents(derivation_id,citations,execution_source) VALUES($1,$2,$3)")
                .bind(&row.id)
                .bind(sqlx::types::Json(&document.citations))
                .bind(sqlx::types::Json(&document.execution_source)).execute(&mut **tx).await.map_err(db::sql_error)?;
        }
        if let Some(image) = &row.generated_image {
            sqlx::query("INSERT INTO spaces_generated_images(derivation_id,intent,style,aspect_ratio,execution_source,revised_prompt) VALUES($1,$2,$3,$4,$5,$6)")
                .bind(&row.id).bind(&image.intent).bind(&image.style).bind(&image.aspect_ratio)
                .bind(sqlx::types::Json(&image.execution_source)).bind(&image.revised_prompt)
                .execute(&mut **tx).await.map_err(db::sql_error)?;
        }
        if let Some(video) = &row.generated_video {
            sqlx::query("INSERT INTO spaces_generated_videos(derivation_id,prompt,aspect_ratio,duration_seconds,execution_source,operation_id) VALUES($1,$2,$3,$4,$5,$6)")
                .bind(&row.id).bind(&video.prompt).bind(&video.aspect_ratio).bind(video.duration_seconds)
                .bind(sqlx::types::Json(&video.execution_source)).bind(&video.operation_id)
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
            | "extract-image-selection"
            | "generate-image-depth"
            | "generate-visualization"
            | "generate-image"
            | "add-video-subtitles"
            | "render-video-edit"
            | "draft-document"
            | "draft-script"
            | "summarize-meeting"
            | "draft-social-post"
            | "convert-document-to-docx"
            | "convert-document-to-pdf"
            | "convert-document-to-txt"
            | "convert-document-to-html"
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
        (
            "draft-document" | "draft-script" | "summarize-meeting" | "draft-social-post",
            Some(metadata),
        ) => metadata.citations.is_array() && metadata.execution_source.validate().is_ok(),
        ("draft-document" | "draft-script" | "summarize-meeting" | "draft-social-post", None) => {
            row.status == "failed"
        }
        (_, None) => true,
        (_, Some(_)) => false,
    };
    let image = match (&row.action_kind[..], &row.generated_image) {
        ("generate-visualization" | "generate-image", Some(metadata)) => {
            !metadata.intent.trim().is_empty()
                && !metadata.aspect_ratio.trim().is_empty()
                && metadata.execution_source.validate().is_ok()
        }
        ("generate-visualization" | "generate-image", None) => row.status == "failed",
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
mod tests;
