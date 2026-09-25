use serde_json::Value;

use crate::spaces::db;

pub(super) async fn append_citation_sources(
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
