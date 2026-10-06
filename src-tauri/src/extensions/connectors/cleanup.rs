use super::super::database;
use std::collections::BTreeSet;
use tauri::AppHandle;

pub(crate) async fn collect(app: &AppHandle) -> Result<(), String> {
    let pool = database::pool(app).await?;
    let retained = super::definitions::list(&pool)
        .await?
        .into_iter()
        .map(|connector| connector.id)
        .collect::<BTreeSet<_>>();
    for account in super::accounts::list(&pool)
        .await?
        .into_iter()
        .filter(|account| !retained.contains(&account.connector_id))
    {
        super::commands::disconnect_connector_account(app.clone(), account.id.clone()).await?;
        sqlx::query("DELETE FROM connector_accounts WHERE id=$1")
            .bind(&account.id)
            .execute(&pool)
            .await
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}
