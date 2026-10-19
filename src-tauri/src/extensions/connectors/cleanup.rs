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
    let mut errors = Vec::new();
    for account in super::accounts::list(&pool)
        .await?
        .into_iter()
        .filter(|account| {
            account.cleanup_pending || !account.enabled || !retained.contains(&account.connector_id)
        })
    {
        let cleanup = async {
            super::commands::disconnect_connector(app.clone(), account.connector_id.clone())
                .await?;
            if !retained.contains(&account.connector_id) {
                sqlx::query("DELETE FROM connector_accounts WHERE connector_id=$1")
                    .bind(&account.connector_id)
                    .execute(&pool)
                    .await
                    .map_err(|error| error.to_string())?;
            }
            Ok::<_, String>(())
        }
        .await;
        if let Err(error) = cleanup {
            errors.push(format!("{}: {error}", account.connector_id));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}
