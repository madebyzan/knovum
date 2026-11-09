use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use tokio::sync::{Mutex, OnceCell};

use crate::{
    ai::{
        capabilities::translation::{self, BatchLineInput},
        models::ModelExecutionTarget,
        tasks::{self, TaskExecutionTarget},
    },
    foundation::app_error::AppError,
    spaces::{
        db,
        runs::{record_usage, AttemptClaim},
    },
};

use super::contracts::{TranslationRunInput, TranslationStepConfiguration};

static COORDINATOR: LazyLock<BatchCoordinator> = LazyLock::new(BatchCoordinator::default);

#[derive(Default)]
pub(crate) struct BatchCoordinator {
    runs: Mutex<HashMap<String, Arc<OnceCell<Result<HashMap<usize, String>, String>>>>>,
}

pub(crate) async fn get_batched_line(
    app: &tauri::AppHandle,
    pool: &sqlx::SqlitePool,
    claim: &AttemptClaim,
    target: &ModelExecutionTarget,
    config: &TranslationStepConfiguration,
) -> Result<Option<String>, AppError> {
    COORDINATOR
        .get_or_translate_line(app, pool, claim, target, config)
        .await
}

impl BatchCoordinator {
    async fn get_or_translate_line(
        &self,
        app: &tauri::AppHandle,
        pool: &sqlx::SqlitePool,
        claim: &AttemptClaim,
        target: &ModelExecutionTarget,
        config: &TranslationStepConfiguration,
    ) -> Result<Option<String>, AppError> {
        let (cell, is_new) = {
            let mut guard = self.runs.lock().await;
            if let Some(existing) = guard.get(&claim.run_id) {
                (Arc::clone(existing), false)
            } else {
                let cell = Arc::new(OnceCell::new());
                guard.insert(claim.run_id.clone(), Arc::clone(&cell));
                (cell, true)
            }
        };

        if is_new {
            let run_id = claim.run_id.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(60)).await;
                COORDINATOR.runs.lock().await.remove(&run_id);
            });
        }

        let result = cell
            .get_or_init(|| async { Self::execute_batch(app, pool, claim, target).await })
            .await;

        match result {
            Ok(map) => Ok(map.get(&config.ordinal).cloned()),
            Err(_) => Ok(None),
        }
    }

    async fn execute_batch(
        app: &tauri::AppHandle,
        pool: &sqlx::SqlitePool,
        claim: &AttemptClaim,
        target: &ModelExecutionTarget,
    ) -> Result<HashMap<usize, String>, String> {
        let snapshot: String =
            sqlx::query_scalar("SELECT input_snapshot FROM spaces_runs WHERE id=?")
                .bind(&claim.run_id)
                .fetch_one(pool)
                .await
                .map_err(db::sql_error)?;

        let input: TranslationRunInput = serde_json::from_str(&snapshot).map_err(db::json_error)?;

        let batch_lines: Vec<BatchLineInput> = input
            .segments
            .iter()
            .enumerate()
            .map(|(ordinal, seg)| BatchLineInput {
                id: ordinal,
                text: seg.text.clone(),
                duration_ms: seg.target_duration_ms,
            })
            .collect();

        let task_input = translation::dubbing_batch_input(
            &batch_lines,
            input.target_language.clone(),
            &input.terms,
        )
        .map_err(|e| e.to_string())?;

        let output = tasks::completion::complete(
            app,
            TaskExecutionTarget::Model(target.clone()),
            task_input,
            claim.cancellation.clone(),
        )
        .await
        .map_err(|e| e.to_string())?;

        if let Some(usage) = &output.usage {
            let _ = record_usage(
                pool,
                &claim.run_id,
                usage.prompt_tokens,
                usage.completion_tokens,
                usage.total_tokens,
            )
            .await;
        }

        translation::parse_batch_output(&output.text)
    }
}
