use sqlx::{Sqlite, Transaction};

use crate::spaces::db;

mod endpoints;
mod validation;

pub(crate) use endpoints::{canonical as canonical_endpoints, is_symmetric as is_symmetric_kind};
pub(crate) use validation::{
    validate_confidence, validate_current_status, validate_endpoints, validate_kind,
    validate_rationale, validate_status,
};

pub async fn exists(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    relation_id: &str,
) -> Result<bool, String> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM spaces_relations WHERE id=$1 AND space_id=$2 AND status <> 'invalidated'",
    )
    .bind(relation_id)
    .bind(space_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(count == 1)
}
