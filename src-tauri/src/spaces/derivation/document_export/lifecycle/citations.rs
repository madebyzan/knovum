use serde_json::Value;

use crate::documents::generation::document::Citation;
use crate::spaces::db;

pub(super) async fn load(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    node_id: &str,
) -> Result<Vec<Citation>, String> {
    let encoded: Option<String> = sqlx::query_scalar(
        "SELECT g.citations FROM spaces_knowledge_derivations d JOIN spaces_grounded_documents g ON g.derivation_id=d.id WHERE d.output_node_id=$1 AND d.status='completed'",
    )
    .bind(node_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let Some(encoded) = encoded else {
        return Ok(Vec::new());
    };
    let citations: Vec<Value> = serde_json::from_str(&encoded).map_err(db::json_error)?;
    citations
        .into_iter()
        .map(|citation| {
            Ok(Citation {
                title: citation["nodeTitle"]
                    .as_str()
                    .ok_or("Citation title is invalid")?
                    .to_owned(),
                quote: citation["quote"]
                    .as_str()
                    .ok_or("Citation quote is invalid")?
                    .to_owned(),
            })
        })
        .collect()
}
