use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::AppHandle;

use super::completion;
use crate::spaces::derivation::{failure, inputs, source_validation};
use crate::spaces::{assets::asset_directory, db, knowledge::contracts};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExportDocumentRequest {
    pub id: String,
    pub space_id: String,
    pub source_node_id: String,
    pub source_revision: String,
    pub output_node_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportDocumentOutput {
    pub node_id: String,
    pub data: Value,
}

pub struct PreparedDocumentExport {
    pub markdown: String,
    pub path: String,
}

pub async fn prepare(
    app: &AppHandle,
    request: &ExportDocumentRequest,
    action: &str,
    engine: &str,
    extension: &str,
) -> Result<PreparedDocumentExport, String> {
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    source_validation::validate(&mut tx, &request.source_node_id, &request.source_revision).await?;
    contracts::validate_same_space_node(
        &mut tx,
        &request.space_id,
        &request.source_node_id,
        &request.output_node_id,
    )
    .await?;
    let source: String = sqlx::query_scalar("SELECT data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND kind='markdown' AND deleted_at IS NULL")
        .bind(&request.source_node_id).bind(&request.space_id).fetch_one(&mut *tx).await.map_err(db::sql_error)?;
    let output: String = sqlx::query_scalar(
        "SELECT data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND kind='file'",
    )
    .bind(&request.output_node_id)
    .bind(&request.space_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    let source: Value = serde_json::from_str(&source).map_err(db::json_error)?;
    let output: Value = serde_json::from_str(&output).map_err(db::json_error)?;
    let path = validated_path(app, &output, extension)?;
    let markdown = source["text"]
        .as_str()
        .filter(|value| !value.trim().is_empty())
        .ok_or("Document export requires non-empty Markdown")?
        .to_owned();
    let markdown = append_citation_sources(&mut tx, &request.source_node_id, markdown).await?;
    let now = crate::clock::unix_time_millis()?;
    sqlx::query("INSERT INTO spaces_knowledge_derivations(id,space_id,output_node_id,action_kind,action_version,engine,status,created_at,updated_at) VALUES($1,$2,$3,$4,1,$5,'running',$6,$6)")
        .bind(&request.id).bind(&request.space_id).bind(&request.output_node_id).bind(action).bind(engine).bind(now)
        .execute(&mut *tx).await.map_err(db::sql_error)?;
    inputs::insert(
        &mut tx,
        &request.id,
        &[inputs::DerivationInput {
            node_id: &request.source_node_id,
            role: "primary",
            source_revision: &request.source_revision,
            revision_basis: inputs::RevisionBasis::Knowledge,
        }],
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(PreparedDocumentExport { markdown, path })
}

async fn append_citation_sources(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    node_id: &str,
    mut markdown: String,
) -> Result<String, String> {
    let encoded: Option<String> = sqlx::query_scalar(
        "SELECT g.citations FROM spaces_knowledge_derivations d JOIN spaces_grounded_documents g ON g.derivation_id=d.id WHERE d.output_node_id=$1 AND d.status='completed'",
    )
    .bind(node_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let Some(encoded) = encoded else {
        return Ok(markdown);
    };
    let citations: Vec<Value> = serde_json::from_str(&encoded).map_err(db::json_error)?;
    if citations.is_empty() {
        return Ok(markdown);
    }
    markdown.push_str("\n\n## Sources\n");
    for (index, citation) in citations.iter().enumerate() {
        let title = citation["nodeTitle"]
            .as_str()
            .ok_or("Citation title is invalid")?;
        let quote = citation["quote"]
            .as_str()
            .ok_or("Citation quote is invalid")?;
        markdown.push_str(&format!("\n- [C{}] **{}** — {}", index + 1, title, quote));
    }
    Ok(markdown)
}

pub async fn complete(
    app: &AppHandle,
    request: &ExportDocumentRequest,
    bytes: &[u8],
    rationale: &'static str,
) -> Result<ExportDocumentOutput, String> {
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    source_validation::validate(&mut tx, &request.source_node_id, &request.source_revision).await?;
    let encoded: String = sqlx::query_scalar(
        "SELECT data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND kind='file'",
    )
    .bind(&request.output_node_id)
    .bind(&request.space_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    let mut data: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
    completion::apply(&mut tx, request, &mut data, bytes, rationale).await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(ExportDocumentOutput {
        node_id: request.output_node_id.clone(),
        data,
    })
}

pub async fn handle_failure(
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

fn validated_path(app: &AppHandle, output: &Value, extension: &str) -> Result<String, String> {
    let path = output["assetPath"]
        .as_str()
        .ok_or("Document output path is unavailable")?;
    let value = std::path::Path::new(path);
    if value.parent() != Some(asset_directory(app)?.as_path())
        || value.extension().and_then(|part| part.to_str()) != Some(extension)
    {
        return Err("Document output path is invalid".into());
    }
    Ok(path.to_owned())
}
