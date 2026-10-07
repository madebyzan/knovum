use std::time::Duration;

use tauri::AppHandle;
use tokio::sync::oneshot;

use crate::{app_error::AppResult, spaces::db};

pub(in crate::spaces::external_conversations::runtime) fn observe(
    app: AppHandle,
    conversation_id: String,
    run_id: String,
    completion: oneshot::Receiver<AppResult<()>>,
    deadline: i64,
) {
    tauri::async_runtime::spawn(async move {
        if let Err(error) = settle(&app, &conversation_id, &run_id, completion, deadline).await {
            log::error!("External Agent session settlement failed: {error}");
        }
    });
}

async fn settle(
    app: &AppHandle,
    conversation_id: &str,
    run_id: &str,
    mut completion: oneshot::Receiver<AppResult<()>>,
    deadline: i64,
) -> Result<(), String> {
    let remaining = Duration::from_millis(
        deadline
            .saturating_sub(crate::clock::unix_time_millis()?)
            .max(0) as u64,
    );
    let (error, deadline_reached) = tokio::select! {
        outcome = &mut completion => match outcome {
            Ok(Ok(())) => return Ok(()),
            Ok(Err(error)) => (error.to_string(), false),
            Err(_) => ("External Agent runtime stopped without a completion result".into(), false),
        },
        _ = tokio::time::sleep(remaining) => {
            let mut error = "External Agent conversation duration budget was exhausted".to_owned();
            if crate::external_agents::conversation::is_active(conversation_id) {
                if let Err(shutdown) = crate::external_agents::conversation::close(conversation_id).await {
                    error.push_str(&format!("; Agent shutdown failed: {shutdown}"));
                }
            }
            (error, true)
        }
    };
    let error = match super::super::super::updates::flush(conversation_id).await {
        Ok(()) => error,
        Err(updates) => format!("{error}; Agent update settlement failed: {updates}"),
    };
    let pool = db::pool(app).await?;
    if super::mark_disconnected(pool, conversation_id, run_id, &error).await? {
        if deadline_reached {
            let _outcome = completion
                .await
                .map_err(|_| "External Agent runtime lost its completion result")?;
        }
        crate::spaces::conversations::extensions::release_run(app, pool, run_id).await?;
    }
    Ok(())
}
