pub(crate) mod contracts;
mod files;
mod repository;
mod subtitles;
pub(crate) mod tracks;

use super::{
    cancellation, publication,
    video_publication::{final_mix::MixOutput, mixing, policy},
};
use crate::{
    app_error::AppError, file_inspection::verify_content_revision, local_resources::tools,
    spaces::db,
};
use contracts::{ExportVideoDubbingAssetRequest, VideoAudioTrack, VideoDeliveryAsset};
use std::{fs, path::Path};
use tauri::AppHandle;

#[tauri::command]
pub async fn list_video_audio_tracks(
    app: AppHandle,
    project_id: String,
) -> Result<Vec<VideoAudioTrack>, AppError> {
    Ok(repository::tracks(db::pool(&app).await?, &project_id).await?)
}

#[tauri::command]
pub async fn export_video_dubbing_asset(
    app: AppHandle,
    request: ExportVideoDubbingAssetRequest,
) -> Result<(), AppError> {
    let lease = cancellation::acquire(&request.job_id)?;
    let destination = Path::new(&request.destination_path).to_owned();
    let extension = match request.asset {
        VideoDeliveryAsset::Srt => "srt",
        VideoDeliveryAsset::Vtt => "vtt",
        _ => "wav",
    };
    files::authorize(&app, &destination, extension)?;
    let pool = db::pool(&app).await?;
    let revision = repository::load(pool, &request.project_id, &request.revision_id).await?;
    let config = revision.config()?;
    let segments = super::repository::load_segments(pool, &request.revision_id).await?;
    let subtitle = match request.asset {
        VideoDeliveryAsset::Srt => Some(subtitles::encode(&segments, false)?),
        VideoDeliveryAsset::Vtt => Some(subtitles::encode(&segments, true)?),
        _ => {
            revision.published_output()?;
            None
        }
    };
    let renders = if subtitle.is_none() {
        repository::renders(pool, &request.revision_id).await?
    } else {
        Vec::new()
    };
    if subtitle.is_none() && renders.len() != segments.len() {
        return Err("Dubbing stem export is missing completed segment renders".into());
    }
    let windows = segments
        .iter()
        .map(|segment| (segment.target_start_ms, segment.target_end_ms))
        .collect::<Vec<_>>();
    let work = files::workspace(
        &app,
        &request.project_id,
        &request.revision_id,
        &request.job_id,
    )?;
    let cancellation = lease.token().clone();
    Ok(crate::blocking::run(move || -> Result<_, String> {
        let delivery = (|| {
            let output = if let Some(subtitle) = subtitle {
                let path = work.join(format!("subtitles.{extension}"));
                fs::write(&path, subtitle).map_err(|error| error.to_string())?;
                path
            } else {
                let ffmpeg = tools::resolve_tool(&app, "KNOVUM_FFMPEG_PATH", tools::ffmpeg_name())?
                    .ok_or("Bundled FFmpeg is unavailable")?;
                let video = policy::verified_source(
                    &app,
                    &config.video.asset_path,
                    &config.video.source_revision,
                )?;
                let background = config
                    .background
                    .as_ref()
                    .map(|source| {
                        policy::verified_source(&app, &source.asset_path, &source.source_revision)
                    })
                    .transpose()?;
                let duration = policy::verified_duration_us(
                    &ffmpeg,
                    &video,
                    "0:v:0",
                    config.video.duration_ms,
                    &cancellation,
                )?;
                let samples = policy::samples_for_duration(duration)?;
                let renders = renders
                    .into_iter()
                    .map(|render| publication::validate_cached(&app, render))
                    .collect::<Result<Vec<_>, _>>()?;
                let timed = mixing::timed_renders(&renders, &windows, samples)?;
                let kind = match request.asset {
                    VideoDeliveryAsset::DialogueWav => MixOutput::Dialogue,
                    VideoDeliveryAsset::BackgroundWav => MixOutput::Background,
                    _ => return Err("Video stem selection is invalid".into()),
                };
                let output = mixing::build_stem(
                    mixing::MixRequest {
                        ffmpeg: &ffmpeg,
                        work: &work,
                        timed: &timed,
                        video_samples: samples,
                        background: background.as_deref(),
                        config: &config,
                        cancellation: &cancellation,
                    },
                    kind,
                )?;
                verify_content_revision(&video, &config.video.source_revision)?;
                if let (Some(path), Some(source)) = (background, &config.background) {
                    verify_content_revision(&path, &source.source_revision)?;
                }
                for render in renders {
                    publication::validate_cached(&app, render)?;
                }
                output
            };
            files::commit(&output, &destination, &cancellation)
        })();
        files::finish(delivery, &work)
    })
    .await?)
}
