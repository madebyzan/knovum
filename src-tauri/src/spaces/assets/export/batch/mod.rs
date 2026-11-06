use std::{fs::OpenOptions, path::PathBuf};

use tauri::AppHandle;

use super::{files, jobs, load, write};
use crate::foundation::{app_error::AppError, scratch_file::ScratchFile};

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExportMember {
    node_id: String,
    file_name: String,
    text: Option<String>,
}

/// One directory grant, one cancellable operation. Existing files are never replaced.
#[tauri::command]
pub async fn export_selection(
    app: AppHandle,
    space_id: String,
    directory_path: String,
    members: Vec<ExportMember>,
    job_id: String,
) -> Result<(), AppError> {
    let (cancellation, _lease) = jobs::begin(&job_id)?;
    let directory = PathBuf::from(directory_path);
    crate::spaces::assets::authorize_destination(&app, &directory)?;
    if members.is_empty() || !directory.is_dir() {
        return Err("An export needs resources and a destination directory".into());
    }
    let mut pending = Vec::with_capacity(members.len());
    for member in members {
        if cancellation.is_cancelled() {
            return Err(AppError::cancelled("The export was cancelled"));
        }
        let payload = load(&app, &space_id, &member.node_id).await?;
        let destination = reserve(&app, &directory, &member.file_name)?;
        let guard = ScratchFile::new(destination.clone());
        if let Some(text) = member.text {
            files::write(&app, &destination, text.as_bytes())?;
        } else {
            write(
                app.clone(),
                &member.node_id,
                payload,
                destination.clone(),
                &cancellation,
            )
            .await?;
        }
        pending.push(guard);
    }
    if cancellation.is_cancelled() {
        return Err(AppError::cancelled("The export was cancelled"));
    }
    for file in pending {
        file.retain();
    }
    Ok(())
}

fn reserve(app: &AppHandle, directory: &std::path::Path, name: &str) -> Result<PathBuf, String> {
    let (stem, extension) = file_parts(name)?;
    for suffix in 0..10_000 {
        let name = if suffix == 0 {
            name.to_owned()
        } else {
            format!("{stem} ({suffix}).{extension}")
        };
        let destination = directory.join(name);
        crate::spaces::assets::authorize_destination(app, &destination)?;
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&destination)
        {
            Ok(_) => return Ok(destination),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("No unused export file name is available".into())
}

fn file_parts(name: &str) -> Result<(&str, &str), String> {
    let path = std::path::Path::new(name);
    if name.is_empty()
        || name.contains(['/', '\\'])
        || path.file_name().and_then(|p| p.to_str()) != Some(name)
    {
        return Err("An export member must have a file name".into());
    }
    let stem = path
        .file_stem()
        .and_then(|p| p.to_str())
        .ok_or("Invalid export name")?;
    let extension = path
        .extension()
        .and_then(|p| p.to_str())
        .ok_or("Missing export format")?;
    Ok((stem, extension))
}

#[cfg(test)]
mod tests;
