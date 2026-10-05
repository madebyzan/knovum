use crate::spaces::speech::contracts::SpeechAudioOutput;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(super) fn part_path(path: &Path) -> PathBuf {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("audio");
    path.with_extension(format!("{extension}.part"))
}

pub(super) fn finish_assembly(
    assembly: Result<SpeechAudioOutput, String>,
    part: &Path,
    work_directory: &Path,
) -> Result<SpeechAudioOutput, String> {
    let mut cleanup_errors = Vec::new();
    if let Err(error) = fs::remove_file(part) {
        if error.kind() != std::io::ErrorKind::NotFound {
            cleanup_errors.push(format!("failed to remove {}: {error}", part.display()));
        }
    }
    if let Err(error) = fs::remove_dir_all(work_directory) {
        if error.kind() != std::io::ErrorKind::NotFound {
            cleanup_errors.push(format!(
                "failed to remove {}: {error}",
                work_directory.display()
            ));
        }
    }
    match (assembly, cleanup_errors.is_empty()) {
        (Ok(output), true) => Ok(output),
        (Ok(_), false) => Err(cleanup_errors.join("; ")),
        (Err(error), true) => Err(error),
        (Err(mut error), false) => {
            error.push_str("; ");
            error.push_str(&cleanup_errors.join("; "));
            Err(error)
        }
    }
}
