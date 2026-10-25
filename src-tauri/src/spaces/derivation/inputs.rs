use serde_json::{json, Value};
use sqlx::{Sqlite, Transaction};

use crate::spaces::db;

use super::input_revision;
pub use super::input_revision::RevisionBasis;

pub struct DerivationInput<'a> {
    pub node_id: &'a str,
    pub role: &'a str,
    pub source_revision: &'a str,
    pub revision_basis: RevisionBasis,
}

pub async fn insert(
    tx: &mut Transaction<'_, Sqlite>,
    derivation_id: &str,
    values: &[DerivationInput<'_>],
) -> Result<(), String> {
    for (position, value) in values.iter().enumerate() {
        validate(value)?;
        let row: Option<(String, String)> = sqlx::query_as(
            "SELECT n.kind,n.data FROM spaces_nodes n JOIN spaces_knowledge_derivations d ON d.space_id=n.space_id WHERE d.id=$1 AND n.id=$2 AND n.deleted_at IS NULL",
        )
        .bind(derivation_id).bind(value.node_id)
        .fetch_optional(&mut **tx).await.map_err(db::sql_error)?;
        let (kind, encoded) = row.ok_or("Derivation input is unavailable")?;
        let data: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
        let (input_revision, input_snapshot) =
            input_revision::resolve(&kind, &data, value.revision_basis)?;
        let base_field = match value.revision_basis {
            RevisionBasis::Knowledge => "knowledgeRevision",
            RevisionBasis::Asset | RevisionBasis::Rendered => "sourceRevision",
        };
        if data.get(base_field).and_then(Value::as_str) != Some(value.source_revision) {
            return Err("Derivation input changed before it was recorded".into());
        }
        let source_snapshot = json!({
            "title": data.get("title").and_then(Value::as_str).ok_or("Derivation input title is unavailable")?,
            "kind": kind,
            "originalUrl": if kind == "website" { data.get("url") } else { data.pointer("/origin/url") },
            "platform": data.pointer("/origin/provider"),
        });
        let inserted = sqlx::query(
            "INSERT INTO spaces_knowledge_derivation_inputs(derivation_id,position,node_id,role,source_revision,revision_basis,input_revision,input_snapshot,source_identity,source_snapshot) \
             VALUES($1,$2,$3,$4,$5,$6,$7,$8,$3,$9)",
        )
        .bind(derivation_id)
        .bind(position as i64)
        .bind(value.node_id)
        .bind(value.role)
        .bind(value.source_revision)
        .bind(value.revision_basis.as_str())
        .bind(input_revision)
        .bind(sqlx::types::Json(&input_snapshot))
        .bind(sqlx::types::Json(&source_snapshot))
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
        if inserted.rows_affected() != 1 {
            return Err("Derivation input is unavailable".into());
        }
    }
    Ok(())
}

pub async fn validate_current(
    tx: &mut Transaction<'_, Sqlite>,
    derivation_id: &str,
) -> Result<(), String> {
    let rows: Vec<(String, String, String, String)> = sqlx::query_as(
        "SELECT n.kind,n.data,i.revision_basis,i.input_revision FROM spaces_knowledge_derivation_inputs i JOIN spaces_knowledge_derivations d ON d.id=i.derivation_id JOIN spaces_nodes n ON n.id=i.node_id AND n.space_id=d.space_id WHERE i.derivation_id=$1 AND n.deleted_at IS NULL ORDER BY i.position",
    )
    .bind(derivation_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let expected: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_knowledge_derivation_inputs WHERE derivation_id=$1",
    )
    .bind(derivation_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    if rows.is_empty() || rows.len() != expected as usize {
        return Err("Derivation input is unavailable".into());
    }
    for (kind, encoded, basis, expected_revision) in rows {
        let data: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
        let (current, _) = input_revision::resolve(&kind, &data, RevisionBasis::parse(&basis)?)?;
        if current != expected_revision {
            return Err("Derivation input changed during processing".into());
        }
    }
    Ok(())
}

fn validate(value: &DerivationInput<'_>) -> Result<(), String> {
    if value.node_id.trim().is_empty() || value.source_revision.trim().is_empty() {
        return Err("Derivation input identity and revision are required".into());
    }
    if !matches!(
        value.role,
        "primary" | "source" | "context" | "style-reference"
    ) {
        return Err("Derivation input role is invalid".into());
    }
    Ok(())
}
