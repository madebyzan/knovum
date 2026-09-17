use sqlx::{Sqlite, Transaction};

use crate::spaces::db;

pub struct DerivationInput<'a> {
    pub node_id: &'a str,
    pub role: &'a str,
    pub source_revision: &'a str,
}

pub async fn insert(
    tx: &mut Transaction<'_, Sqlite>,
    derivation_id: &str,
    values: &[DerivationInput<'_>],
) -> Result<(), String> {
    if values.is_empty() {
        return Err("Knowledge derivation requires at least one input".into());
    }
    for (position, value) in values.iter().enumerate() {
        validate(value)?;
        let inserted = sqlx::query(
            "INSERT INTO spaces_knowledge_derivation_inputs(derivation_id,position,node_id,role,source_revision,source_identity,source_snapshot) \
             SELECT $1,$2,id,$4,$5,id,json_object('title',json_extract(data,'$.title'),'kind',kind,'originalUrl',CASE WHEN kind='website' THEN json_extract(data,'$.url') ELSE json_extract(data,'$.origin.url') END,'platform',json_extract(data,'$.origin.provider')) \
             FROM spaces_nodes WHERE id=$3 AND deleted_at IS NULL",
        )
        .bind(derivation_id)
        .bind(position as i64)
        .bind(value.node_id)
        .bind(value.role)
        .bind(value.source_revision)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
        if inserted.rows_affected() != 1 {
            return Err("Derivation input is unavailable".into());
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
