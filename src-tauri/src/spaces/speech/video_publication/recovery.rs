use std::{fs, path::Path};

use tauri::{AppHandle, Runtime};

use super::super::{contracts::SpeechAudioOutput, publication::hash_file};
use super::SAMPLE_RATE;
use crate::spaces::{assets, db};

#[derive(sqlx::FromRow)]
struct ReusableVideoOutput {
    output_asset_path: Option<String>,
    output_sha256: Option<String>,
    output_sample_rate: Option<i64>,
    output_sample_count: Option<i64>,
}

pub(in crate::spaces::speech) fn reconcile_workspaces<R: Runtime>(
    app: &AppHandle<R>,
) -> Result<(), String> {
    let speech = assets::asset_directory(app)?.join("speech");
    if !speech.exists() {
        return Ok(());
    }
    for project in fs::read_dir(speech).map_err(|error| error.to_string())? {
        let project = project.map_err(|error| error.to_string())?.path();
        if !managed_directory(&project) {
            continue;
        }
        for revision in fs::read_dir(project).map_err(|error| error.to_string())? {
            let revision = revision.map_err(|error| error.to_string())?.path();
            if !managed_directory(&revision) {
                continue;
            }
            for entry in fs::read_dir(revision).map_err(|error| error.to_string())? {
                let path = entry.map_err(|error| error.to_string())?.path();
                let name = path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or("");
                let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
                if [
                    "video-assembly-",
                    "assembly-",
                    "chapter-assembly-",
                    "loudness-",
                ]
                .iter()
                .any(|prefix| {
                    name.strip_prefix(prefix)
                        .is_some_and(|id| uuid::Uuid::parse_str(id).is_ok())
                }) && metadata.is_dir()
                    && !metadata.file_type().is_symlink()
                {
                    fs::remove_dir_all(path).map_err(|error| error.to_string())?;
                } else if name.starts_with("full-")
                    && [".wav.part", ".mp3.part", ".m4b.part", ".mp4.part"]
                        .iter()
                        .any(|suffix| name.ends_with(suffix))
                    && metadata.is_file()
                    && !metadata.file_type().is_symlink()
                {
                    fs::remove_file(path).map_err(|error| error.to_string())?;
                }
            }
        }
    }
    Ok(())
}

fn managed_directory(path: &Path) -> bool {
    uuid::Uuid::parse_str(
        path.file_name()
            .and_then(|value| value.to_str())
            .unwrap_or(""),
    )
    .is_ok()
        && fs::symlink_metadata(path)
            .is_ok_and(|metadata| metadata.is_dir() && !metadata.file_type().is_symlink())
}

pub(super) async fn reusable_output<R: Runtime>(
    app: &AppHandle<R>,
    pool: &sqlx::SqlitePool,
    project_id: &str,
    revision_id: &str,
    digest: &str,
    expected_samples: u64,
) -> Result<Option<SpeechAudioOutput>, String> {
    let row: Option<ReusableVideoOutput> = sqlx::query_as(
        "SELECT output_asset_path,output_sha256,output_sample_rate,output_sample_count \
         FROM spaces_speech_project_revisions WHERE id=? AND project_id=?",
    )
    .bind(revision_id)
    .bind(project_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?;
    let Some(ReusableVideoOutput {
        output_asset_path: Some(path),
        output_sha256: Some(sha256),
        output_sample_rate: Some(rate),
        output_sample_count: Some(samples),
    }) = row
    else {
        return Ok(None);
    };
    let source = Path::new(&path);
    let prefix = format!("full-{digest}-");
    let valid_name = source
        .file_name()
        .and_then(|value| value.to_str())
        .and_then(|name| {
            name.strip_prefix(&prefix)
                .and_then(|value| value.strip_suffix(".mp4"))
        })
        .is_some_and(|attempt| uuid::Uuid::parse_str(attempt).is_ok());
    if !valid_name
        || rate != i64::from(SAMPLE_RATE)
        || samples
            != i64::try_from(expected_samples)
                .map_err(|_| "Video sample count exceeds SQLite range")?
    {
        return Ok(None);
    }
    let Ok(source) = assets::authorize_spaces_asset(app, source) else {
        return Ok(None);
    };
    let Ok(actual) = hash_file(&source) else {
        return Ok(None);
    };
    if actual != sha256 {
        return Ok(None);
    }
    Ok(Some(SpeechAudioOutput {
        path: source.to_string_lossy().into_owned(),
        sha256,
        sample_rate: SAMPLE_RATE,
        sample_count: u64::try_from(samples).map_err(|_| "Stored video sample count is invalid")?,
    }))
}
