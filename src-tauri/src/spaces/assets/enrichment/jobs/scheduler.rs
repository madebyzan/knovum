use std::{collections::HashSet, time::Duration};

use serde::{de::IntoDeserializer, Deserialize, Serialize};
use tauri::{ipc::Channel, AppHandle, Emitter};
use tokio::task::JoinSet;

use crate::{ai::tasks::TaskTargetSelection, foundation::app_error::AppError};

use super::{
    execution::RunEnrichmentJobRequest,
    job::{EnrichmentJobStatus, EnrichmentJobType},
    persistence,
};

#[derive(Clone, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase")]
pub(crate) struct EnrichmentJobUpdate {
    job_id: String,
    space_id: String,
    node_id: String,
    job_type: EnrichmentJobType,
    created_at: i64,
    status: EnrichmentJobStatus,
    progress: Option<super::super::super::transcription::MediaTranscriptionProgress>,
    #[ts(type = "unknown | null")]
    data: Option<serde_json::Value>,
    error: Option<String>,
}

pub(crate) fn start(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        if let Err(error) = run(app.clone()).await {
            log::error!("Enrichment scheduler stopped: {error}");
            if let Err(emit_error) = app.emit("space-enrichment-error", error) {
                log::error!("Could not publish scheduler failure: {emit_error}");
            }
        }
    });
}

type NodeIdentity = (String, String);
type EnrichmentWorkers = JoinSet<Result<NodeIdentity, String>>;

async fn run(app: AppHandle) -> Result<(), String> {
    super::reconcile_interrupted(&app).await?;
    let pool = crate::spaces::db::pool(&app).await?;
    let mut active = HashSet::new();
    let mut workers = JoinSet::new();
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    loop {
        tokio::select! {
            joined = workers.join_next(), if !workers.is_empty() => {
                let node = joined.ok_or("Enrichment worker disappeared")?
                    .map_err(|error| format!("Enrichment worker panicked: {error}"))??;
                active.remove(&node);
            }
            _ = interval.tick() => dispatch(&app, pool, &mut active, &mut workers).await?,
        }
    }
}

async fn dispatch(
    app: &AppHandle,
    pool: &'static sqlx::SqlitePool,
    active: &mut HashSet<NodeIdentity>,
    workers: &mut EnrichmentWorkers,
) -> Result<(), String> {
    let mut readiness = VisionReadiness::default();
    for job in persistence::list_actionable(pool, None).await? {
        if !matches!(
            job.status,
            EnrichmentJobStatus::Pending | EnrichmentJobStatus::AwaitingConfiguration
        ) {
            continue;
        }
        let node = (job.space_id.clone(), job.node_id.clone());
        if active.contains(&node) || active.len() >= 32 {
            continue;
        }
        let selection = match execution_choice(app, &job, &mut readiness).await {
            Ok(selection) => selection,
            Err(error) if error.is_not_configured() => {
                persistence::defer_for_configuration(
                    pool,
                    &job.id,
                    error.to_string(),
                    crate::foundation::clock::unix_time_millis()?,
                )
                .await?;
                continue;
            }
            Err(error) => {
                persistence::set_status(
                    pool,
                    &job.id,
                    "'pending','awaiting_configuration'",
                    "failed",
                    Some(error.to_string()),
                    crate::foundation::clock::unix_time_millis()?,
                )
                .await?;
                continue;
            }
        };
        let target = match selection {
            Some(target) => Some(persistence::freeze_target(pool, &job.id, &target).await?),
            None => None,
        };
        active.insert(node.clone());
        let app = app.clone();
        workers.spawn(async move {
            execute(&app, pool, job, target).await?;
            Ok(node)
        });
    }
    Ok(())
}

/// The executor a job runs on, once it can actually run there. Only visual analysis is routed; a
/// target frozen by an earlier attempt is proven again, because its connection may have changed.
async fn execution_choice(
    app: &AppHandle,
    job: &super::job::EnrichmentJobState,
    readiness: &mut VisionReadiness,
) -> Result<Option<TaskTargetSelection>, AppError> {
    if job.job_type != EnrichmentJobType::AnalyzeVisual {
        return Ok(None);
    }
    let selection = match &job.execution_target {
        Some(target) => target.clone(),
        None => crate::ai::settings::default_selection(
            app,
            crate::ai::contracts::RoutingPurpose::VisualUnderstanding,
        )?,
    };
    readiness.prove(app, &selection).await?;
    Ok(Some(selection))
}

/// Readiness proven once per dispatch pass: every job waiting on one executor shares one answer,
/// so a backlog of pictures does not read the same credential once per picture every second.
#[derive(Default)]
struct VisionReadiness(Vec<(TaskTargetSelection, Result<(), AppError>)>);

impl VisionReadiness {
    async fn prove(
        &mut self,
        app: &AppHandle,
        selection: &TaskTargetSelection,
    ) -> Result<(), AppError> {
        if let Some((_, known)) = self.0.iter().find(|(proven, _)| proven == selection) {
            return known.clone();
        }
        let known = crate::ai::settings::ready_task(
            app,
            selection.clone(),
            crate::ai::contracts::ExecutionCapability::VisionUnderstanding,
        )
        .await
        .map(|_| ());
        self.0.push((selection.clone(), known.clone()));
        known
    }
}

async fn execute(
    app: &AppHandle,
    pool: &sqlx::SqlitePool,
    job: super::job::EnrichmentJobState,
    target: Option<crate::ai::tasks::TaskTargetSelection>,
) -> Result<(), String> {
    let update = EnrichmentJobUpdate {
        job_id: job.id.clone(),
        space_id: job.space_id.clone(),
        node_id: job.node_id.clone(),
        job_type: job.job_type,
        created_at: job.created_at,
        status: EnrichmentJobStatus::Running,
        progress: None,
        data: None,
        error: None,
    };
    publish_update(app, &update);
    let progress_app = app.clone();
    let progress_update = update.clone();
    let progress = Channel::new(move |message| {
        let mut next = progress_update.clone();
        next.progress = Some(message.deserialize()?);
        publish_update(&progress_app, &next);
        Ok(())
    });
    let request = RunEnrichmentJobRequest {
        job_id: job.id,
        space_id: job.space_id,
        node_id: job.node_id,
        job_type: job.job_type,
        source_revision: job.source_revision,
        vision_target: target,
    };
    let execution = super::execution::execute_job(app.clone(), request, progress).await;
    let mut completed = update;
    match execution {
        Ok(output) => completed.data = Some(output.data),
        Err(error) => completed.error = Some(error.to_string()),
    }
    // The row is the outcome: it holds a cancellation that raced the return, and whether a failure
    // was configuration the job now waits on rather than a failed run.
    let status: Option<String> =
        sqlx::query_scalar("SELECT status FROM spaces_enrichment_jobs WHERE id=$1")
            .bind(&completed.job_id)
            .fetch_optional(pool)
            .await
            .map_err(crate::spaces::db::sql_error)?;
    let Some(status) = status else {
        return Ok(());
    };
    completed.status = EnrichmentJobStatus::deserialize(status.into_deserializer())
        .map_err(crate::spaces::db::json_error)?;
    publish_update(app, &completed);
    Ok(())
}

fn publish_update(app: &AppHandle, update: &EnrichmentJobUpdate) {
    if let Err(error) = app.emit("space-enrichment-update", update) {
        log::error!("Could not publish enrichment observation: {error}");
    }
}
