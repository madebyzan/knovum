use arrow_array::{Float32Array, RecordBatch, StringArray};
use futures::TryStreamExt;
use lancedb::query::{ExecutableQuery, QueryBase};
use serde_json::Value;
use tauri::{AppHandle, Runtime};

use crate::{ai::capabilities::embedding, spaces::db};

use super::{
    current_chunks, embedding_contract, lancedb_store::open_knowledge_table,
    predicates::escape_sql_literal, schema::VECTOR_COLUMN, validation::validate_query_vector,
};

pub(crate) async fn embed_query<R: Runtime>(
    app: &AppHandle<R>,
    query: &str,
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<Vec<f32>, String> {
    let query = query.trim();
    if query.is_empty() {
        return Err("Semantic search query is required".to_owned());
    }
    let resources = crate::local_resources::preparation::acquire_verified_models(
        app,
        crate::local_resources::models::catalog::TEXT_EMBEDDING_MODELS,
        cancellation,
    )
    .await?;
    let formatted = embedding_contract::format_query(query);
    let response = embedding::embed(vec![formatted], &resources, cancellation).await?;
    response
        .vectors
        .into_iter()
        .next()
        .ok_or_else(|| "Embedding provider returned no query vector".to_owned())
}

pub(crate) async fn search_semantic_vector<R: Runtime>(
    app: &AppHandle<R>,
    space_id: &str,
    query: &[f32],
    node_ids: Option<&[String]>,
    exclude_node_id: Option<&str>,
    limit: i64,
) -> Result<Vec<Value>, String> {
    validate_query_vector(query, limit)?;
    let table = open_knowledge_table(app).await?;
    let filter = build_filter(
        space_id,
        embedding_contract::active_model_id(),
        node_ids,
        exclude_node_id,
    );
    let batches = table
        .vector_search(query.to_vec())
        .map_err(|error| format!("Failed to prepare LanceDB vector search: {error}"))?
        .column(VECTOR_COLUMN)
        .only_if(filter)
        .limit(limit as usize)
        .execute()
        .await
        .map_err(|error| format!("Failed to execute LanceDB vector search: {error}"))?
        .try_collect::<Vec<RecordBatch>>()
        .await
        .map_err(|error| format!("Failed to read LanceDB vector search results: {error}"))?;
    let values = batches_to_values(&batches)?;
    let pool = db::pool(app).await?;
    current_chunks::filter(pool, space_id, values).await
}

fn build_filter(
    space_id: &str,
    model: &str,
    node_ids: Option<&[String]>,
    exclude_node_id: Option<&str>,
) -> String {
    let mut filter = format!(
        "space_id = '{}' AND model = '{}'",
        escape_sql_literal(space_id),
        escape_sql_literal(model)
    );
    if let Some(ids) = node_ids.filter(|ids| !ids.is_empty()) {
        let quoted_ids = ids
            .iter()
            .map(|id| format!("'{}'", escape_sql_literal(id)))
            .collect::<Vec<_>>()
            .join(",");
        filter.push_str(" AND node_id IN (");
        filter.push_str(&quoted_ids);
        filter.push(')');
    }
    if let Some(id) = exclude_node_id {
        filter.push_str(" AND node_id != '");
        filter.push_str(&escape_sql_literal(id));
        filter.push('\'');
    }
    filter
}

fn batches_to_values(batches: &[RecordBatch]) -> Result<Vec<Value>, String> {
    let mut values = Vec::new();
    for batch in batches {
        let chunk_ids = utf8(batch, "chunk_id")?;
        let node_ids = utf8(batch, "node_id")?;
        let node_kinds = utf8(batch, "node_kind")?;
        let node_titles = utf8(batch, "node_title")?;
        let chunk_kinds = utf8(batch, "chunk_kind")?;
        let source_revisions = utf8(batch, "source_revision")?;
        let locators = utf8(batch, "locator")?;
        let texts = utf8(batch, "text")?;
        let metadata = utf8(batch, "metadata")?;
        let distances = distance(batch)?;
        for row in 0..batch.num_rows() {
            let distance = distances.value(row) as f64;
            let score = 1.0 / (1.0 + distance.max(0.0));
            values.push(serde_json::json!({
                "chunkId": chunk_ids.value(row),
                "nodeId": node_ids.value(row),
                "nodeKind": node_kinds.value(row),
                "nodeTitle": node_titles.value(row),
                "chunkKind": chunk_kinds.value(row),
                "metadata": serde_json::from_str::<Value>(metadata.value(row)).map_err(crate::spaces::db::json_error)?,
                "sourceRevision": source_revisions.value(row),
                "locator": serde_json::from_str::<Value>(locators.value(row)).map_err(crate::spaces::db::json_error)?,
                "snippet": texts.value(row),
                "text": texts.value(row),
                "score": score,
            }));
        }
    }
    Ok(values)
}

fn utf8<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a StringArray, String> {
    let index = batch
        .schema()
        .index_of(name)
        .map_err(|error| format!("LanceDB result missing column {name}: {error}"))?;
    batch
        .column(index)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| format!("LanceDB result column {name} has unexpected type"))
}

fn distance(batch: &RecordBatch) -> Result<&Float32Array, String> {
    let index = batch
        .schema()
        .index_of("_distance")
        .map_err(|error| format!("LanceDB result missing column _distance: {error}"))?;
    batch
        .column(index)
        .as_any()
        .downcast_ref::<Float32Array>()
        .ok_or_else(|| "LanceDB result _distance column has unexpected type".to_owned())
}
