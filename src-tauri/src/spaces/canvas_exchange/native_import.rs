use super::{contract::JsonCanvasDocumentV1, import::JsonCanvasImportReceipt};
use crate::spaces::{
    assets::import::commit::AssetImportProgress,
    db,
    structure::{receipts, repository},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{ipc::Channel, AppHandle};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ImportRequest {
    pub(super) space_id: String,
    pub(super) source_path: String,
    pub(super) mutation_id: String,
    pub(super) base_revision: i64,
    pub(super) note_title: String,
    pub(super) section_title: String,
}

pub(super) async fn execute(
    app: AppHandle,
    request: ImportRequest,
    progress: Channel<AssetImportProgress>,
) -> Result<JsonCanvasImportReceipt, String> {
    let (cancellation, _lease) = super::import_control::begin(&request.mutation_id)?;
    progress
        .send(AssetImportProgress {
            stage: "preparing",
            bytes_copied: 0,
            total_bytes: None,
        })
        .map_err(|error| error.to_string())?;
    if request.note_title.trim().is_empty() || request.section_title.trim().is_empty() {
        return Err("JSON Canvas import titles cannot be empty".into());
    }
    let read_app = app.clone();
    let source = PathBuf::from(&request.source_path);
    let (document, directory, raw) = crate::blocking::run(move || {
        crate::spaces::assets::authorize_source(&read_app, &source)?;
        let source = source.canonicalize().map_err(|error| error.to_string())?;
        if std::fs::metadata(&source)
            .map_err(|error| error.to_string())?
            .len()
            > 32 * 1024 * 1024
        {
            return Err("JSON Canvas document exceeds 32 MiB".into());
        }
        let raw = std::fs::read_to_string(&source).map_err(|error| error.to_string())?;
        let document: JsonCanvasDocumentV1 =
            serde_json::from_str(&raw).map_err(|error| error.to_string())?;
        super::validation::validate(&document)?;
        if document.nodes.is_empty() {
            return Err("JSON Canvas document contains no nodes".into());
        }
        let directory = source
            .parent()
            .ok_or("JSON Canvas source has no directory")?
            .to_path_buf();
        Ok::<_, String>((document, directory, raw))
    })
    .await?;
    let digest = receipts::digest(&(&request, &raw))?;
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    if let Some(import_receipt) =
        receipts::find_typed(&mut tx, &request.mutation_id, &digest).await?
    {
        tx.commit().await.map_err(db::sql_error)?;
        return Ok(import_receipt);
    }
    let (snapshot, revision) = repository::load_in_transaction(&mut tx, &request.space_id).await?;
    if revision != request.base_revision {
        return Err("space-edit-revision-conflict: Space edit revision conflict".into());
    }
    tx.commit().await.map_err(db::sql_error)?;
    super::import_control::check(&cancellation)?;
    let worker_app = app.clone();
    let worker_cancellation = cancellation.clone();
    let (prepared, acquired) = crate::blocking::run(move || {
        let mut acquired = Vec::new();
        let prepared = super::conversion::prepare(
            &request,
            document,
            snapshot,
            &mut super::acquisition::ImportResources {
                app: &worker_app,
                directory: &directory,
                progress: &progress,
                acquired: &mut acquired,
                cancellation: &worker_cancellation,
            },
        );
        Ok::<_, String>((prepared, acquired))
    })
    .await?;
    let outcome = match prepared {
        Ok(prepared) => {
            super::import::import_nodes(app.clone(), prepared, digest, &cancellation).await
        }
        Err(error) => Err(error),
    };
    match outcome {
        Ok(import_receipt) => Ok(import_receipt),
        Err(cause) => {
            crate::spaces::assets::commands::remove_assets(app, acquired)
                .await
                .map_err(|error| {
                    format!("JSON Canvas import failed: {cause}; asset cleanup failed: {error}")
                })?;
            Err(cause)
        }
    }
}
