use std::{fs, path::Path};

use tauri::{AppHandle, Runtime};

use crate::spaces::{assets, db};

pub(in crate::spaces::speech) async fn cleanup_audiobook_checkpoints<R: Runtime>(
    app: &AppHandle<R>,
    pool: &sqlx::SqlitePool,
    revision_id: &str,
) -> Result<(), String> {
    let paths: Vec<String> = sqlx::query_scalar(
        "SELECT asset_path FROM spaces_speech_chapter_checkpoints WHERE project_revision_id=?",
    )
    .bind(revision_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    for stored in paths {
        let path = Path::new(&stored);
        match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
                let path = assets::authorize_spaces_asset(app, path)?;
                fs::remove_file(path).map_err(|error| error.to_string())?;
            }
            Ok(_) => return Err("Audiobook checkpoint is not a managed regular file".into()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.to_string()),
        }
    }
    let mut tx = db::begin_write(pool).await?;
    sqlx::query("DELETE FROM spaces_speech_chapter_checkpoints WHERE project_revision_id=?")
        .bind(revision_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    sqlx::query("DELETE FROM spaces_speech_loudness_checkpoints WHERE project_revision_id=?")
        .bind(revision_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)
}

pub(super) fn paths(mut error: String, files: &[&Path], directories: &[&Path]) -> String {
    for path in files {
        if let Err(cleanup) = fs::remove_file(path) {
            if cleanup.kind() != std::io::ErrorKind::NotFound {
                error.push_str(&format!("; failed to remove {}: {cleanup}", path.display()));
            }
        }
    }
    for path in directories {
        if let Err(cleanup) = fs::remove_dir_all(path) {
            if cleanup.kind() != std::io::ErrorKind::NotFound {
                error.push_str(&format!("; failed to remove {}: {cleanup}", path.display()));
            }
        }
    }
    error
}

pub(super) fn checkpoint_workspace(
    directory: &Path,
    digest: &str,
    checkpoint_path: &str,
) -> Result<(), String> {
    let Some(file_name) = Path::new(checkpoint_path)
        .file_name()
        .and_then(|value| value.to_str())
    else {
        return Ok(());
    };
    let prefix = format!("chapter-{digest}-");
    let Some(attempt_id) = file_name
        .strip_prefix(&prefix)
        .and_then(|value| value.strip_suffix(".wav"))
    else {
        return Ok(());
    };
    if uuid::Uuid::parse_str(attempt_id).is_err() {
        return Ok(());
    }
    remove_directory(&directory.join(format!("chapter-assembly-{attempt_id}")))
}

pub(super) fn loudness_workspaces(directory: &Path) -> Result<(), String> {
    for entry in fs::read_dir(directory).map_err(|error| error.to_string())? {
        let path = entry.map_err(|error| error.to_string())?.path();
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        let Some(attempt_id) = name.strip_prefix("loudness-") else {
            continue;
        };
        if uuid::Uuid::parse_str(attempt_id).is_err() {
            continue;
        }
        let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
        if metadata.is_dir() && !metadata.file_type().is_symlink() {
            fs::remove_dir_all(path).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn remove_directory(path: &Path) -> Result<(), String> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}
