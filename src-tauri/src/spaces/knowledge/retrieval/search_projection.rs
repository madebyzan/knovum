use arrow_array::{Float32Array, RecordBatch, StringArray};
use serde_json::Value;

pub(super) fn project_search_matches(batches: &[RecordBatch]) -> Result<Vec<Value>, String> {
    let mut values = Vec::new();
    for batch in batches {
        let space_ids = utf8(batch, "space_id")?;
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
                "spaceId": space_ids.value(row),
                "chunkId": chunk_ids.value(row),
                "nodeId": node_ids.value(row),
                "nodeKind": node_kinds.value(row),
                "nodeTitle": node_titles.value(row),
                "chunkKind": chunk_kinds.value(row),
                "metadata": serde_json::from_str::<Value>(metadata.value(row))
                    .map_err(crate::spaces::db::json_error)?,
                "sourceRevision": source_revisions.value(row),
                "locator": serde_json::from_str::<Value>(locators.value(row))
                    .map_err(crate::spaces::db::json_error)?,
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
