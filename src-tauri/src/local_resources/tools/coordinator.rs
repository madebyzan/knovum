use crate::local_resources::preparation::shared_installation::Registry;
use std::sync::{Arc, OnceLock};
use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

use super::{
    catalog::{self, MediaImportToolKind},
    filesystem, PreparedMediaImportTools, ToolPreparationProgress,
};
use crate::local_resources::preparation::{
    contracts::{ResourcePreparationProgress, ResourcePreparationStage, ResourcePreparationTarget},
    requests::{self, PreparationRequest},
    RESOURCE_PREPARATION_CANCELLED,
};

pub(crate) async fn prepare_media_import_tools<R: Runtime>(
    app: &AppHandle<R>,
    request_cancellation: &CancellationToken,
    progress: &mut impl FnMut(ToolPreparationProgress),
) -> Result<PreparedMediaImportTools, String> {
    require_active(request_cancellation)?;
    if let Some(prepared) = resolve_prepared(app)? {
        progress(ToolPreparationProgress {
            stage: ResourcePreparationStage::Ready,
            downloaded: 0,
            total: Some(0),
        });
        return Ok(prepared);
    }
    static INSTALLATIONS: OnceLock<Arc<Registry<ToolPreparationProgress>>> = OnceLock::new();
    let registry = INSTALLATIONS.get_or_init(Registry::new);
    let handle = app.clone();
    let mut waiter = registry
        .join(
            request_cancellation,
            ToolPreparationProgress {
                stage: ResourcePreparationStage::Queued,
                downloaded: 0,
                total: None,
            },
            move |installation| async move {
                install_missing(&handle, &installation.cancellation, &mut |update| {
                    installation.publish(update)
                })
                .await
            },
        )
        .await?;
    let mut latest = ToolPreparationProgress {
        stage: ResourcePreparationStage::Queued,
        downloaded: 0,
        total: None,
    };
    waiter
        .wait(request_cancellation, &mut |update| {
            latest = update.clone();
            progress(update);
        })
        .await?;
    require_active(request_cancellation)?;
    let prepared = resolve_prepared(app)?.ok_or_else(|| {
        "Media import tool installation completed without verified executables".to_string()
    })?;
    latest.stage = ResourcePreparationStage::Ready;
    progress(latest);
    Ok(prepared)
}

pub(super) async fn install_missing<R: Runtime>(
    app: &AppHandle<R>,
    cancellation: &CancellationToken,
    progress: &mut impl FnMut(ToolPreparationProgress),
) -> Result<(), String> {
    let descriptors = [
        catalog::descriptor(MediaImportToolKind::YtDlp)?,
        catalog::descriptor(MediaImportToolKind::Deno)?,
    ];
    let mut completed = 0_u64;
    for descriptor in descriptors {
        if filesystem::verified_installation_path(app, &descriptor)?.is_some() {
            continue;
        }
        let mut transferred = 0_u64;
        super::installer::install_tool(app, &descriptor, cancellation, &mut |mut update| {
            transferred = update.downloaded;
            update.downloaded = completed.saturating_add(update.downloaded);
            update.total = update.total.map(|value| completed.saturating_add(value));
            progress(update);
        })
        .await?;
        completed = completed.saturating_add(transferred);
    }
    Ok(())
}

pub(crate) async fn prepare_media_import_tools_for_job<R: Runtime>(
    app: &AppHandle<R>,
    job_id: &str,
    title: &str,
    business_cancellation: &CancellationToken,
    progress: &mut impl FnMut(ResourcePreparationProgress),
) -> Result<PreparedMediaImportTools, String> {
    if job_id.trim().is_empty() || title.trim().is_empty() {
        return Err(
            "Media import tool preparation requires a job identifier and title".to_string(),
        );
    }
    let request = requests::begin(job_id, title, ResourcePreparationTarget::MediaImportTools)?;
    prepare_media_import_tools_for_request(app, request, business_cancellation, progress).await
}

pub(crate) async fn prepare_media_import_tools_for_request<R: Runtime>(
    app: &AppHandle<R>,
    request: PreparationRequest,
    business_cancellation: &CancellationToken,
    progress: &mut impl FnMut(ResourcePreparationProgress),
) -> Result<PreparedMediaImportTools, String> {
    let prepared_tools = {
        let mut on_update = |update: ToolPreparationProgress| {
            let update = ResourcePreparationProgress {
                stage: update.stage,
                downloaded: update.downloaded,
                total: update.total,
                percent: update
                    .total
                    .filter(|value| *value > 0)
                    .map(|value| ((update.downloaded.saturating_mul(100) / value).min(100)) as u8),
            };
            request.update(update.clone());
            progress(update);
        };
        tokio::select! {
            biased;
            _ = business_cancellation.cancelled() => {
                request.cancellation().cancel();
                Err(RESOURCE_PREPARATION_CANCELLED.to_string())
            }
            prepared_tools = prepare_media_import_tools(app, request.cancellation(), &mut on_update) => prepared_tools,
        }
    };
    request.finish(prepared_tools.as_ref().err())?;
    prepared_tools
}

fn resolve_prepared<R: Runtime>(
    app: &AppHandle<R>,
) -> Result<Option<PreparedMediaImportTools>, String> {
    let yt_dlp = super::resolve_yt_dlp(app)?;
    let deno = super::resolve_deno(app)?;
    Ok(match (yt_dlp, deno) {
        (Some(yt_dlp), Some(deno)) => Some(PreparedMediaImportTools { yt_dlp, deno }),
        _ => None,
    })
}

fn require_active(cancellation: &CancellationToken) -> Result<(), String> {
    if cancellation.is_cancelled() {
        Err(RESOURCE_PREPARATION_CANCELLED.to_string())
    } else {
        Ok(())
    }
}
