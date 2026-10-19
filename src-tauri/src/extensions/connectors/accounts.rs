//! Connector authorizations.
//!
//! A service is connected once. An account is therefore identified by the connector it authorizes,
//! which is why nothing here takes an account identity of its own.

use super::super::{contracts::ConnectorAccount, database};
use sqlx::SqlitePool;

pub(crate) async fn list(pool: &SqlitePool) -> Result<Vec<ConnectorAccount>, String> {
    sqlx::query_scalar::<_, String>("SELECT document FROM connector_accounts ORDER BY connector_id")
        .fetch_all(pool)
        .await
        .map_err(|error| error.to_string())?
        .iter()
        .map(|document| database::decode(document))
        .collect()
}

/// The account that authorizes a connector, if the user has connected it.
pub(crate) async fn of_connector(
    pool: &SqlitePool,
    connector_id: &str,
) -> Result<Option<ConnectorAccount>, String> {
    let document: Option<String> =
        sqlx::query_scalar("SELECT document FROM connector_accounts WHERE connector_id=$1")
            .bind(connector_id)
            .fetch_optional(pool)
            .await
            .map_err(|error| error.to_string())?;
    document.as_deref().map(database::decode).transpose()
}

pub(crate) async fn load(
    pool: &SqlitePool,
    connector_id: &str,
) -> Result<ConnectorAccount, String> {
    of_connector(pool, connector_id)
        .await?
        .ok_or_else(|| "connector-not-connected".into())
}

pub(super) async fn save(pool: &SqlitePool, account: &ConnectorAccount) -> Result<(), String> {
    sqlx::query("INSERT INTO connector_accounts(connector_id,document) VALUES($1,$2) ON CONFLICT(connector_id) DO UPDATE SET document=excluded.document")
        .bind(&account.connector_id).bind(sqlx::types::Json(account)).execute(pool).await.map_err(|error| error.to_string())?;
    Ok(())
}

pub(super) async fn set_status(
    pool: &SqlitePool,
    connector_id: &str,
    enabled: bool,
    cleanup_pending: bool,
) -> Result<(), String> {
    let updated=sqlx::query("UPDATE connector_accounts SET document=json_set(document,'$.enabled',json($1),'$.cleanupPending',json($2)) WHERE connector_id=$3").bind(enabled.to_string()).bind(cleanup_pending.to_string()).bind(connector_id).execute(pool).await.map_err(|error|error.to_string())?;
    if updated.rows_affected() != 1 {
        return Err("connector-not-connected".into());
    }
    Ok(())
}

pub(super) async fn begin_authorization(
    pool: &SqlitePool,
    account: &ConnectorAccount,
) -> Result<(), String> {
    let updated = sqlx::query("UPDATE connector_accounts SET document=json_set(document,'$.enabled',json('false')) WHERE connector_id=$1 AND document=$2")
        .bind(&account.connector_id).bind(sqlx::types::Json(account)).execute(pool).await.map_err(|error| error.to_string())?;
    if updated.rows_affected() != 1 {
        return Err("connector-account-revoked-during-authorization".into());
    }
    Ok(())
}

/// Publishes a new authorization. The connector's connections lose their approved tools, because
/// what the user approved was approved for the authorization that has just been replaced.
pub(super) async fn commit_authorization(
    pool: &SqlitePool,
    account: &ConnectorAccount,
    previous: &ConnectorAccount,
) -> Result<(), String> {
    let mut tx = pool.begin().await.map_err(|error| error.to_string())?;
    let updated = sqlx::query(
        "UPDATE connector_accounts SET document=$1 WHERE connector_id=$2 AND document=$3",
    )
    .bind(sqlx::types::Json(account))
    .bind(&account.connector_id)
    .bind(sqlx::types::Json(previous))
    .execute(&mut *tx)
    .await
    .map_err(|error| error.to_string())?;
    if updated.rows_affected() != 1 {
        return Err("connector-account-revoked-during-authorization".into());
    }
    sqlx::query("UPDATE connections SET document=json_set(document,'$.enabled',json('false'),'$.tools',json('[]'),'$.revision',$1) WHERE json_extract(document,'$.connectorId')=$2")
        .bind(uuid::Uuid::new_v4().to_string()).bind(&account.connector_id).execute(&mut *tx).await.map_err(|error|error.to_string())?;
    tx.commit().await.map_err(|error| error.to_string())
}

pub(in crate::extensions) async fn require_current(
    pool: &SqlitePool,
    frozen: &ConnectorAccount,
) -> Result<(), String> {
    let current = load(pool, &frozen.connector_id).await?;
    if !current.enabled || current.cleanup_pending || current.revision != frozen.revision {
        return Err("connector-account-authorization-revised-or-revoked".into());
    }
    Ok(())
}

/// What a connection requires of its connector's authorization before it may be used.
pub(in crate::extensions) fn validate_binding(
    connector: &super::super::contracts::ConnectorDefinition,
    account: &ConnectorAccount,
) -> Result<(), String> {
    validate_identity(connector, account)?;
    if !account.enabled || account.cleanup_pending {
        return Err("connector-account-not-authorized".into());
    }
    Ok(())
}

pub(super) fn validate_identity(
    connector: &super::super::contracts::ConnectorDefinition,
    account: &ConnectorAccount,
) -> Result<(), String> {
    if account.connector_id != connector.id
        || account.resource
            != super::super::mcp::endpoint::resource(connector.configured_endpoint()?)
        || account.authorization != connector.authorization
    {
        return Err("connector-account-binding-mismatch".into());
    }
    Ok(())
}
