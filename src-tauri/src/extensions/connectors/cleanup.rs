use super::super::{database, mcp::connections};
use std::collections::BTreeSet;
use tauri::AppHandle;

pub(crate) async fn collect(app: &AppHandle) -> Result<(), String> {
    let pool = database::pool(app).await?;
    let retained = connections::list(&pool)
        .await?
        .into_iter()
        .map(|connection| connection.connector_id)
        .collect::<BTreeSet<_>>();
    for connector in super::accounts::definitions(&pool)
        .await?
        .into_iter()
        .filter(|connector| !retained.contains(&connector.id))
    {
        for account in super::accounts::list(&pool)
            .await?
            .into_iter()
            .filter(|account| account.connector_id == connector.id)
        {
            super::commands::disconnect_connector_account(app.clone(), account.id.clone()).await?;
            sqlx::query("DELETE FROM connector_accounts WHERE id=$1")
                .bind(&account.id)
                .execute(&pool)
                .await
                .map_err(|error| error.to_string())?;
        }
        sqlx::query("DELETE FROM connectors WHERE id=$1")
            .bind(connector.id)
            .execute(&pool)
            .await
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}
