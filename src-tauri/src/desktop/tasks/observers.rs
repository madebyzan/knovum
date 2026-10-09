use tauri::AppHandle;

pub(super) fn install(app: &AppHandle) {
    let resources_app = app.clone();
    tauri::async_runtime::spawn(async move {
        let mut changes = crate::local_resources::preparation::requests::subscribe();
        loop {
            let projection = crate::local_resources::preparation::requests::desktop_tasks();
            match projection.and_then(|tasks| super::publish(&resources_app, "resources", tasks)) {
                Ok(()) => {}
                Err(error) => log::error!("resource desktop task projection failed: {error}"),
            }
            if changes.changed().await.is_err() {
                return;
            }
        }
    });
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(error) = crate::spaces::await_runtime(&app).await {
            log::error!("desktop task observation blocked by Space startup: {error}");
            return;
        }
        let mut changes = crate::spaces::runs::subscribe_to_changes();
        loop {
            let projection = async {
                let pool = crate::spaces::db::pool(&app).await?;
                let mut tasks = crate::spaces::runs::desktop_tasks(pool).await?;
                tasks.extend(crate::spaces::assets::enrichment::jobs::desktop_tasks(pool).await?);
                super::publish(&app, "spaces", tasks)
            }
            .await;
            if let Err(error) = projection {
                log::error!("Space desktop task projection failed: {error}");
            }
            if changes.changed().await.is_err() {
                return;
            }
            // Coalesce committed changes; execution and recovery remain feature-owned.
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    });
}
