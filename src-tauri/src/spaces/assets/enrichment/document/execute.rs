use serde_json::Value;
use tauri::{ipc::Channel, AppHandle};
use tokio_util::sync::CancellationToken;

use crate::app_error::AppError;
use crate::spaces::assets::enrichment::{commit, jobs::RunEnrichmentJobRequest};
use crate::spaces::knowledge::model::document_coverage::{
    validate_analysis, DocumentCoverage, VisualState,
};
use crate::{
    ai::tasks::TaskExecutionTarget,
    spaces::assets::{
        authorize_spaces_asset, enrichment::ensure_active,
        transcription::MediaTranscriptionProgress,
    },
};

pub(in crate::spaces) async fn analyze(
    app: &AppHandle,
    request: &RunEnrichmentJobRequest,
    payload: Value,
    target: TaskExecutionTarget,
    progress: &Channel<MediaTranscriptionProgress>,
    cancellation: &CancellationToken,
) -> Result<Value, AppError> {
    let mut report = DocumentCoverage::from_node_content(&payload)?;
    super::checkpoint::restore(app, request, &mut report).await?;
    let source = payload
        .get("assetPath")
        .and_then(Value::as_str)
        .ok_or("Document source is missing")?;
    let source = authorize_spaces_asset(app, std::path::Path::new(source))?;
    let fingerprint = source_fingerprint(&source)?;
    let checked_source = source.clone();
    let revision = report.source_revision.clone();
    crate::blocking::run(move || -> Result<(), AppError> {
        if crate::file_inspection::content_revision(&checked_source)? != revision {
            return Err("Document source bytes changed before analysis".into());
        }
        Ok(())
    })
    .await?;
    let mut text_width = report
        .occurrences
        .iter()
        .filter_map(|value| value.analysis.as_ref())
        .map(|value| {
            value.description.encode_utf16().count() + value.detected_text.encode_utf16().count()
        })
        .sum::<usize>();
    let total = report.occurrences.len();
    for index in 0..total {
        ensure_active(cancellation)?;
        require_unchanged_source(&source, &fingerprint)?;
        if matches!(
            report.occurrences[index].state,
            VisualState::Completed | VisualState::Unsupported
        ) {
            continue;
        }
        if report.occurrences[index].resource_id.is_none() {
            return Err("Document resources must be materialized before visual analysis".into());
        }
        let analyzed_document =
            super::analysis::analyze_occurrence(app, &report, index, target.clone(), cancellation)
                .await;
        ensure_active(cancellation)?;
        require_unchanged_source(&source, &fingerprint)?;
        match analyzed_document {
            Ok(analysis) => {
                let width = analysis.description.encode_utf16().count()
                    + analysis.detected_text.encode_utf16().count();
                let validation = validate_analysis(&analysis).and_then(|_| {
                    if text_width + width > 4 * 1024 * 1024 {
                        Err("Document visual text exceeds 4 Mi UTF-16 units".into())
                    } else {
                        Ok(())
                    }
                });
                if let Err(error) = validation {
                    fail_occurrence(
                        app,
                        request,
                        &mut report,
                        (index, &error.to_string()),
                        (&payload["documentCoverage"], cancellation),
                    )
                    .await?;
                    return Err(error.into());
                }
                text_width += width;
                report.occurrences[index].analysis = Some(analysis);
                report.occurrences[index].state = VisualState::Completed;
                report.occurrences[index].error = None;
            }
            Err(error) => {
                fail_occurrence(
                    app,
                    request,
                    &mut report,
                    (index, &error.to_string()),
                    (&payload["documentCoverage"], cancellation),
                )
                .await?;
                return Err(error);
            }
        }
        super::checkpoint::save(app, request, &report, index).await?;
        let _ = progress.send(MediaTranscriptionProgress::stage(
            "document_visual",
            ((index + 1) * 100 / total) as u8,
        ));
    }
    report.validate()?;
    let payload = publish(
        app,
        request,
        &report,
        &payload["documentCoverage"],
        cancellation,
    )
    .await?;
    super::checkpoint::clear(app, &request.job_id).await?;
    Ok(payload)
}

async fn fail_occurrence(
    app: &AppHandle,
    request: &RunEnrichmentJobRequest,
    report: &mut DocumentCoverage,
    (index, error): (usize, &str),
    (expected, cancellation): (&Value, &CancellationToken),
) -> Result<(), AppError> {
    let occurrence = &mut report.occurrences[index];
    occurrence.analysis = None;
    occurrence.state = VisualState::Failed;
    occurrence.error = Some(crate::documents::extraction::text::bounded_utf16_text(
        error, 16_000,
    ));
    super::checkpoint::save(app, request, report, index).await?;
    // Publish completed knowledge once on failure; retries still resume the durable job state.
    report.validate()?;
    publish(app, request, report, expected, cancellation).await?;
    Ok(())
}

fn source_fingerprint(source: &std::path::Path) -> Result<(u64, std::time::SystemTime), AppError> {
    let metadata = std::fs::metadata(source).map_err(|error| error.to_string())?;
    Ok((
        metadata.len(),
        metadata.modified().map_err(|error| error.to_string())?,
    ))
}

fn require_unchanged_source(
    source: &std::path::Path,
    fingerprint: &(u64, std::time::SystemTime),
) -> Result<(), AppError> {
    if &source_fingerprint(source)? != fingerprint {
        return Err("Document source changed during analysis".into());
    }
    Ok(())
}

async fn publish(
    app: &AppHandle,
    request: &RunEnrichmentJobRequest,
    report: &DocumentCoverage,
    expected: &Value,
    cancellation: &CancellationToken,
) -> Result<Value, AppError> {
    let mut fields = serde_json::Map::new();
    fields.insert(
        "documentCoverage".into(),
        serde_json::to_value(report).map_err(|error| error.to_string())?,
    );
    commit::publish(
        app,
        &request.space_id,
        &request.node_id,
        commit::PublicationGuard {
            source_revision: Some(&report.source_revision),
            job_id: Some(&request.job_id),
            media_knowledge: None,
            document_coverage: Some(expected),
            cancellation: Some(cancellation),
        },
        fields,
        None,
    )
    .await
}
