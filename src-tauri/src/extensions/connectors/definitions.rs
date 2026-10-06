use super::super::{contracts::ConnectorDefinition, database};
use sqlx::{SqliteConnection, SqlitePool};

pub(crate) async fn list(pool: &SqlitePool) -> Result<Vec<ConnectorDefinition>, String> {
    sqlx::query_scalar::<_, String>("SELECT document FROM connectors ORDER BY id")
        .fetch_all(pool)
        .await
        .map_err(|error| error.to_string())?
        .iter()
        .map(|document| database::decode(document))
        .collect()
}

pub(crate) async fn load(pool: &SqlitePool, id: &str) -> Result<ConnectorDefinition, String> {
    let document: String = sqlx::query_scalar("SELECT document FROM connectors WHERE id=$1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|error| error.to_string())?
        .ok_or("connector-definition-not-found")?;
    database::decode(&document)
}

pub(crate) async fn require_current(
    pool: &SqlitePool,
    frozen: &ConnectorDefinition,
) -> Result<(), String> {
    let current = load(pool, &frozen.id).await?;
    if current.revision != frozen.revision {
        return Err("connector-definition-revised".into());
    }
    Ok(())
}

pub(in crate::extensions) async fn replace_on(
    tx: &mut SqliteConnection,
    definition: &ConnectorDefinition,
    previous: Option<&ConnectorDefinition>,
) -> Result<(), String> {
    if let Some(previous) = previous {
        let current: String = sqlx::query_scalar(
            "SELECT json_extract(document,'$.revision') FROM connectors WHERE id=$1",
        )
        .bind(&definition.id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|error| error.to_string())?;
        if current != previous.revision {
            return Err("connector-definition-changed-refresh-before-saving".into());
        }
        let access_changed = previous.endpoint != definition.endpoint
            || previous.authorization != definition.authorization;
        if access_changed {
            let documents: Vec<String> = sqlx::query_scalar("SELECT document FROM connector_accounts WHERE json_extract(document,'$.connectorId')=$1")
                .bind(&definition.id).fetch_all(&mut *tx).await.map_err(|error| error.to_string())?;
            for document in documents {
                let mut account: super::super::contracts::ConnectorAccount =
                    database::decode(&document)?;
                account.enabled = false;
                account.cleanup_pending = true;
                account.revision = uuid::Uuid::new_v4().to_string();
                if let Some(endpoint) = &definition.endpoint {
                    account.resource = super::super::mcp::endpoint::resource(endpoint);
                }
                account.authorization = definition.authorization.clone();
                account.issuer = None;
                account.scopes.clear();
                sqlx::query("UPDATE connector_accounts SET document=$1 WHERE id=$2")
                    .bind(database::encode(&account)?)
                    .bind(&account.id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|error| error.to_string())?;
            }
            sqlx::query("UPDATE connections SET document=json_set(document,'$.enabled',json('false'),'$.accountId',NULL,'$.tools',json('[]')) WHERE json_extract(document,'$.connectorId')=$1")
                .bind(&definition.id).execute(&mut *tx).await.map_err(|error| error.to_string())?;
        }
        sqlx::query("UPDATE connections SET document=json_set(document,'$.revision',$1) WHERE json_extract(document,'$.connectorId')=$2")
            .bind(uuid::Uuid::new_v4().to_string()).bind(&definition.id).execute(&mut *tx).await.map_err(|error| error.to_string())?;
    }
    sqlx::query("INSERT INTO connectors(id,document) VALUES($1,$2) ON CONFLICT(id) DO UPDATE SET document=excluded.document")
        .bind(&definition.id).bind(database::encode(definition)?).execute(tx).await.map_err(|error| error.to_string())?;
    Ok(())
}

pub(in crate::extensions) async fn delete_on(
    tx: &mut SqliteConnection,
    id: &str,
) -> Result<(), String> {
    sqlx::query("DELETE FROM connections WHERE json_extract(document,'$.connectorId')=$1")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|error| error.to_string())?;
    sqlx::query("DELETE FROM connectors WHERE id=$1")
        .bind(id)
        .execute(tx)
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}
