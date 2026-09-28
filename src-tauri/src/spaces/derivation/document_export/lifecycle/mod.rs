mod citations;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::AppHandle;

use super::completion;
use crate::documents::generation::document::DocumentSource;
use crate::spaces::db;
use crate::spaces::derivation::{failure, inputs, pending_output, source_validation};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExportDocumentRequest {
    pub id: String,
    pub space_id: String,
    pub source_node_id: String,
    pub source_revision: String,
    pub output_node_id: String,
    pub expected_edit_revision: i64,
    pub target_parent_id: Option<String>,
    pub position: crate::spaces::structure::Point,
    pub size: crate::spaces::structure::Size,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportDocumentOutput {
    pub node_id: String,
    pub data: Value,
}

pub(super) struct PreparedDocumentExport {
    pub source: DocumentSource,
    pub path: String,
}

pub(super) async fn prepare(
    app: &AppHandle,
    request: &ExportDocumentRequest,
    action: &str,
    engine: &str,
    extension: &str,
) -> Result<PreparedDocumentExport, String> {
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    let path = crate::spaces::derivation::output_paths::reserve(app, "document", extension)?;
    let prepared =
        prepare_in_transaction(&mut tx, request, action, engine, extension, &path).await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(prepared)
}

pub(super) async fn prepare_in_transaction(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    request: &ExportDocumentRequest,
    action: &str,
    engine: &str,
    extension: &str,
    path: &str,
) -> Result<PreparedDocumentExport, String> {
    source_validation::validate(
        tx,
        &request.space_id,
        &request.source_node_id,
        &request.source_revision,
        inputs::RevisionBasis::Knowledge,
    )
    .await?;
    let source: String = sqlx::query_scalar("SELECT data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND kind='markdown' AND deleted_at IS NULL")
        .bind(&request.source_node_id).bind(&request.space_id).fetch_one(&mut **tx).await.map_err(db::sql_error)?;
    let source: Value = serde_json::from_str(&source).map_err(db::json_error)?;
    let title = source["title"]
        .as_str()
        .ok_or("Document title is invalid")?
        .to_owned();
    let markdown = source["text"]
        .as_str()
        .ok_or("Document Markdown is invalid")?
        .to_owned();
    if title.trim().is_empty() && markdown.trim().is_empty() {
        return Err("Document export requires a title or Markdown content".into());
    }
    let citations = citations::load(tx, &request.source_node_id).await?;
    let source = DocumentSource {
        title,
        markdown,
        citations,
    };
    let output_title = format!("{}.{}", source.title, extension);
    let mut data = serde_json::json!({"kind":"file","title":output_title,"fileName":output_title,"assetPath":path,"processingState":"processing"});
    crate::spaces::knowledge::indexing::revision::apply(&mut data)?;
    crate::spaces::nodes::insert_many_in_transaction(
        tx,
        crate::spaces::nodes::contracts::InsertNodesRequest {
            space_id: request.space_id.clone(),
            mutation_id: format!("{}:output", request.id),
            expected_edit_revision: request.expected_edit_revision,
            target_parent_id: request.target_parent_id.clone(),
            nodes: vec![crate::spaces::nodes::contracts::InsertNode {
                id: request.output_node_id.clone(),
                kind: "file".into(),
                initial_x: request.position.x,
                initial_y: request.position.y,
                width: request.size.width,
                height: request.size.height,
                sizing_mode: crate::spaces::structure::SizingMode::Frame,
                appearance: None,
                file_subpath: None,
                data,
                source_chunks: vec![],
            }],
        },
    )
    .await?;
    let now = crate::clock::unix_time_millis()?;
    sqlx::query("INSERT INTO spaces_knowledge_derivations(id,space_id,output_node_id,action_kind,action_version,engine,status,created_at,updated_at) VALUES($1,$2,$3,$4,1,$5,'running',$6,$6)")
        .bind(&request.id).bind(&request.space_id).bind(&request.output_node_id).bind(action).bind(engine).bind(now)
        .execute(&mut **tx).await.map_err(db::sql_error)?;
    inputs::insert(
        tx,
        &request.id,
        &[inputs::DerivationInput {
            node_id: &request.source_node_id,
            role: "primary",
            source_revision: &request.source_revision,
            revision_basis: inputs::RevisionBasis::Knowledge,
        }],
    )
    .await?;
    Ok(PreparedDocumentExport {
        source,
        path: path.to_owned(),
    })
}

pub(super) async fn complete(
    app: &AppHandle,
    request: &ExportDocumentRequest,
    bytes: &[u8],
    rationale: &'static str,
) -> Result<ExportDocumentOutput, String> {
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    source_validation::validate(
        &mut tx,
        &request.space_id,
        &request.source_node_id,
        &request.source_revision,
        inputs::RevisionBasis::Knowledge,
    )
    .await?;
    let encoded: String = sqlx::query_scalar(
        "SELECT data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND kind='file'",
    )
    .bind(&request.output_node_id)
    .bind(&request.space_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    let mut data: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
    pending_output::validate_unpublished_asset(&data)?;
    completion::apply(&mut tx, request, &mut data, bytes, rationale).await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(ExportDocumentOutput {
        node_id: request.output_node_id.clone(),
        data,
    })
}

pub(super) async fn handle_failure(
    app: &AppHandle,
    request: &ExportDocumentRequest,
    path: &str,
    export_error: &str,
) -> String {
    let cleanup_error = match tokio::fs::remove_file(path).await {
        Ok(()) => None,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => Some(format!("output cleanup failed: {error}")),
    };
    let persistence_error = failure::fail_message(app, &request.id, export_error)
        .await
        .err()
        .map(|error| format!("failure status update failed: {error}"));
    [
        Some(export_error.to_owned()),
        cleanup_error,
        persistence_error,
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join("; ")
}
