use std::path::{Path, PathBuf};

use sqlx::{Sqlite, Transaction};
use tokio_util::sync::CancellationToken;

use super::publication::dimensions;
use crate::{
    ai::capabilities::agent_artifacts,
    spaces::{assets, db, nodes::contracts::InsertNode, structure},
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum PathRole {
    Asset,
    Preview,
    Embedded,
}

impl PathRole {
    fn as_str(self) -> &'static str {
        match self {
            Self::Asset => "asset",
            Self::Preview => "preview",
            Self::Embedded => "embedded",
        }
    }

    fn decode(value: &str) -> Result<Self, String> {
        match value {
            "asset" => Ok(Self::Asset),
            "preview" => Ok(Self::Preview),
            "embedded" => Ok(Self::Embedded),
            _ => Err("Invalid staged Agent output path role".into()),
        }
    }
}

pub(super) async fn stage(
    app: &tauri::AppHandle,
    pool: &sqlx::SqlitePool,
    run_id: &str,
    workspace: &Path,
) -> Result<Vec<InsertNode>, String> {
    let workspace = workspace.to_path_buf();
    let artifacts = crate::blocking::run({
        let workspace = workspace.clone();
        move || agent_artifacts::collect(&workspace).map_err(|error| error.to_string())
    })
    .await?;
    let mut outputs = Vec::with_capacity(artifacts.len());
    for artifact in artifacts {
        let prepared = crate::blocking::run({
            let app = app.clone();
            let workspace = workspace.clone();
            move || {
                let source = agent_artifacts::resolve_collected(&workspace, &artifact)
                    .map_err(|error| error.to_string())?;
                assets::import::prepare::prepare_owned_asset(&app, source, CancellationToken::new())
            }
        })
        .await;
        let output = match prepared {
            Ok(prepared) => stage_prepared(app, pool, run_id, prepared).await,
            Err(error) => Err(error),
        };
        let output = match output {
            Ok(output) => output,
            Err(error) => return fail_stage(app, pool, run_id, error).await,
        };
        outputs.push(output);
    }
    Ok(outputs)
}

async fn stage_prepared(
    app: &tauri::AppHandle,
    pool: &sqlx::SqlitePool,
    run_id: &str,
    prepared: assets::import::prepare::PreparedSpaceImport,
) -> Result<InsertNode, String> {
    let main_path = prepared.asset.path.as_deref().map(PathBuf::from);
    if let Some(path) = &main_path {
        if let Err(error) = record_path(pool, run_id, path, PathRole::Asset).await {
            return match prepared.job_id.as_deref() {
                Some(job_id) => match assets::import::commit::discard_owned_import(job_id) {
                    Ok(()) => Err(error),
                    Err(cleanup) => {
                        Err(format!("{error}; pending import cleanup failed: {cleanup}"))
                    }
                },
                None => Err(error),
            };
        }
    }
    let asset = match prepared.job_id {
        Some(job_id) => {
            crate::blocking::run(move || assets::import::commit::copy_owned_import(&job_id)).await?
        }
        None => prepared.asset,
    };
    let reserved_preview = assets::enrichment::reserve_import_preview(app, &asset)?;
    let finalized = crate::blocking::run({
        let app = app.clone();
        let pool = pool.clone();
        let run_id = run_id.to_owned();
        let preview = reserved_preview.clone();
        let runtime = tokio::runtime::Handle::current();
        move || {
            let mut claim_preview = |path: &Path| {
                runtime
                    .block_on(record_path(&pool, &run_id, path, PathRole::Preview))
                    .map(|_| ())
            };
            let mut claim_embedded = |path: &Path| {
                runtime
                    .block_on(record_path(&pool, &run_id, path, PathRole::Embedded))
                    .map(|_| ())
            };
            assets::enrichment::finalize_import(
                &app,
                asset,
                preview.as_deref(),
                &CancellationToken::new(),
                &mut claim_preview,
                &mut claim_embedded,
            )
        }
    })
    .await?;
    let kind = finalized
        .data
        .get("kind")
        .and_then(serde_json::Value::as_str)
        .ok_or("Agent output has no node kind")?
        .to_owned();
    let (width, height) = dimensions(&kind);
    Ok(InsertNode {
        id: uuid::Uuid::new_v4().to_string(),
        kind,
        initial_x: 0.0,
        initial_y: 0.0,
        width,
        height,
        sizing_mode: structure::SizingMode::Content,
        appearance: None,
        file_subpath: None,
        data: finalized.data,
        source_chunks: finalized.source_chunks,
    })
}

async fn fail_stage<T>(
    app: &tauri::AppHandle,
    pool: &sqlx::SqlitePool,
    run_id: &str,
    error: String,
) -> Result<T, String> {
    match cleanup(app, pool, run_id).await {
        Ok(()) => Err(error),
        Err(cleanup_error) => Err(format!(
            "{error}; staged output cleanup failed: {cleanup_error}"
        )),
    }
}

async fn record_path(
    pool: &sqlx::SqlitePool,
    run_id: &str,
    path: impl AsRef<Path>,
    role: PathRole,
) -> Result<bool, String> {
    let path = path
        .as_ref()
        .to_str()
        .ok_or("Managed Agent output path is not valid UTF-8")?;
    let inserted = sqlx::query(
        "INSERT INTO spaces_external_output_staging(run_id,path,role,created_at)
         SELECT $1,$2,$3,$4 FROM spaces_runs
         WHERE id=$1 AND cancel_requested_at IS NULL AND state<>'cancelled'
         ON CONFLICT(run_id,path) DO NOTHING",
    )
    .bind(run_id)
    .bind(path)
    .bind(role.as_str())
    .bind(crate::clock::unix_time_millis()?)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    if inserted.rows_affected() == 1 {
        return Ok(true);
    }
    let active: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_runs WHERE id=$1 AND cancel_requested_at IS NULL AND state<>'cancelled'",
    )
    .bind(run_id)
    .fetch_one(pool)
    .await
    .map_err(db::sql_error)?;
    if active == 0 {
        return Err("Agent output owner is cancelled or no longer exists".into());
    }
    let recorded: String = sqlx::query_scalar(
        "SELECT role FROM spaces_external_output_staging WHERE run_id=$1 AND path=$2",
    )
    .bind(run_id)
    .bind(path)
    .fetch_one(pool)
    .await
    .map_err(db::sql_error)?;
    if recorded != role.as_str() {
        return Err("Managed Agent output path has conflicting ownership roles".into());
    }
    Ok(false)
}

pub(super) async fn cleanup(
    app: &tauri::AppHandle,
    pool: &sqlx::SqlitePool,
    run_id: &str,
) -> Result<(), String> {
    // Reference checks, file removal and receipt retirement share the write lock. Another Run
    // cannot claim or publish a shared extracted resource between the check and removal.
    let mut tx = db::begin_write(pool).await?;
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT path,role FROM spaces_external_output_staging WHERE run_id=$1 ORDER BY CASE role WHEN 'preview' THEN 0 WHEN 'embedded' THEN 1 ELSE 2 END,path",
    )
    .bind(run_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    let removable = removable_paths(&mut tx, run_id, &rows).await?;
    for (path, role) in rows {
        if removable.contains(&path) {
            remove_managed_path(app, Path::new(&path), PathRole::decode(&role)?)?;
        }
        sqlx::query("DELETE FROM spaces_external_output_staging WHERE run_id=$1 AND path=$2")
            .bind(run_id)
            .bind(path)
            .execute(&mut *tx)
            .await
            .map_err(db::sql_error)?;
    }
    tx.commit().await.map_err(db::sql_error)
}

async fn removable_paths(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    rows: &[(String, String)],
) -> Result<std::collections::BTreeSet<String>, String> {
    let candidates = rows.iter().map(|(path, _)| path.clone()).collect();
    let mut removable: std::collections::BTreeSet<_> =
        assets::unreferenced_paths_in_transaction(tx, candidates)
            .await?
            .into_iter()
            .collect();
    let shared: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT path FROM spaces_external_output_staging WHERE run_id<>$1",
    )
    .bind(run_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    for path in shared {
        removable.remove(&path);
    }
    Ok(removable)
}

fn remove_managed_path(app: &tauri::AppHandle, path: &Path, role: PathRole) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    let canonical = match role {
        PathRole::Preview => crate::spaces::previews::authorize_generated_preview(app, path)?,
        PathRole::Asset | PathRole::Embedded => assets::authorize_spaces_asset(app, path)?,
    };
    if role == PathRole::Asset {
        crate::spaces::previews::remove_generated(app, &canonical)?;
    }
    std::fs::remove_file(canonical).map_err(|error| error.to_string())
}

pub(super) async fn release(tx: &mut Transaction<'_, Sqlite>, run_id: &str) -> Result<(), String> {
    sqlx::query("DELETE FROM spaces_external_output_staging WHERE run_id=$1")
        .bind(run_id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    Ok(())
}

#[cfg(test)]
mod tests;
