use std::{collections::BTreeSet, fs, path::Path};

use sqlx::SqlitePool;

use crate::spaces::assets;

pub(in crate::spaces::speech) async fn reclaim_unreferenced(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    candidates: BTreeSet<String>,
) -> Result<(), String> {
    let removable = assets::unreferenced_paths(pool, candidates).await?;
    let mut failures = Vec::new();
    for path in removable {
        let candidate = Path::new(&path);
        match fs::symlink_metadata(candidate) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                failures.push(format!("{path}: {error}"));
                continue;
            }
            Ok(_) => {}
        }
        match assets::authorize_spaces_asset(app, candidate) {
            Ok(authorized) => {
                if let Err(error) = fs::remove_file(authorized) {
                    if error.kind() != std::io::ErrorKind::NotFound {
                        failures.push(format!("{path}: {error}"));
                    }
                }
            }
            Err(error) => failures.push(format!("{path}: {error}")),
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("; "))
    }
}
