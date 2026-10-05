use crate::spaces::assets;
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use tokio_util::sync::CancellationToken;

pub(super) fn authorize(
    app: &tauri::AppHandle,
    destination: &Path,
    extension: &str,
) -> Result<(), String> {
    assets::authorize_destination(app, destination)?;
    if !destination.is_absolute()
        || destination.extension().and_then(|value| value.to_str()) != Some(extension)
    {
        return Err(format!(
            "Video delivery requires a .{extension} destination"
        ));
    }
    if destination.exists()
        && !fs::symlink_metadata(destination)
            .map_err(|error| error.to_string())?
            .is_file()
    {
        return Err("Video delivery destination is not a regular file".into());
    }
    Ok(())
}
pub(super) fn workspace(
    app: &tauri::AppHandle,
    project_id: &str,
    revision_id: &str,
    job_id: &str,
) -> Result<PathBuf, String> {
    for id in [project_id, revision_id, job_id] {
        uuid::Uuid::parse_str(id).map_err(|_| "Video delivery identity is invalid")?;
    }
    let directory = assets::asset_directory(app)?
        .join("speech")
        .join(project_id)
        .join(revision_id);
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let work = directory.join(format!("video-delivery-{job_id}"));
    fs::create_dir(&work).map_err(|error| error.to_string())?;
    Ok(work)
}
pub(super) fn commit(
    source: &Path,
    destination: &Path,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let part = destination.with_file_name(format!(".knovum-export-{}.part", uuid::Uuid::new_v4()));
    let result = (|| {
        let mut input = fs::File::open(source).map_err(|error| error.to_string())?;
        let mut output = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&part)
            .map_err(|error| error.to_string())?;
        let mut buffer = [0_u8; 1024 * 1024];
        loop {
            if cancellation.is_cancelled() {
                return Err("cancelled".into());
            }
            let count = input.read(&mut buffer).map_err(|error| error.to_string())?;
            if count == 0 {
                break;
            }
            output
                .write_all(&buffer[..count])
                .map_err(|error| error.to_string())?;
        }
        output.sync_all().map_err(|error| error.to_string())?;
        drop(output);
        if cancellation.is_cancelled() {
            return Err("cancelled".into());
        }
        fs::rename(&part, destination).map_err(|error| error.to_string())
    })();
    if part.exists() {
        fs::remove_file(part).map_err(|cleanup| {
            format!("Export failed; temporary file cleanup failed: {cleanup}")
        })?;
    }
    result
}
pub(super) fn finish(result: Result<(), String>, work: &Path) -> Result<(), String> {
    match fs::remove_dir_all(work) {
        Ok(()) => result,
        Err(error) => Err(format!(
            "{}; delivery workspace cleanup failed: {error}",
            result.err().unwrap_or_else(|| "Export completed".into())
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cancelled_export_preserves_existing_destination_and_removes_its_partial_file() {
        let work =
            std::env::temp_dir().join(format!("knovum-export-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&work).unwrap();
        let source = work.join("source.wav");
        let destination = work.join("existing.wav");
        fs::write(&source, b"new output").unwrap();
        fs::write(&destination, b"existing output").unwrap();
        let cancelled = CancellationToken::new();
        cancelled.cancel();
        assert_eq!(
            commit(&source, &destination, &cancelled).unwrap_err(),
            "cancelled"
        );
        assert_eq!(fs::read(&destination).unwrap(), b"existing output");
        assert_eq!(fs::read_dir(&work).unwrap().count(), 2);
        commit(&source, &destination, &CancellationToken::new()).unwrap();
        assert_eq!(fs::read(&destination).unwrap(), b"new output");
        fs::remove_dir_all(work).unwrap();
    }
}
