pub(crate) mod contracts;
mod db;
mod repository;
#[cfg(test)]
mod tests;

use chrono::NaiveDate;
use tauri::{AppHandle, Runtime};

pub use contracts::UsagePurpose;

use crate::ai::contracts::{ModelTurnResponse, TokenUsage};
use crate::app_error::{AppError, AppResult};

pub async fn initialize<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    db::initialize(app).await
}

pub async fn record_turn(purpose: UsagePurpose, response: &ModelTurnResponse) -> AppResult<()> {
    if let Some(usage) = response.usage.as_ref() {
        record(purpose, &response.provider, &response.model, usage).await?;
    }
    Ok(())
}

pub async fn record(
    purpose: UsagePurpose,
    provider: &str,
    model: &str,
    usage: &TokenUsage,
) -> AppResult<()> {
    repository::insert(
        db::initialized_pool().map_err(AppError::internal)?,
        purpose,
        provider,
        model,
        usage,
    )
    .await
    .map_err(AppError::internal)
}

pub fn from_provider_usage(usage: genai::chat::Usage) -> Option<TokenUsage> {
    let input = u32::try_from(usage.prompt_tokens?).ok()?;
    let output = u32::try_from(usage.completion_tokens?).ok()?;
    Some(TokenUsage {
        prompt_tokens: input,
        completion_tokens: output,
        total_tokens: usage
            .total_tokens
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(input.saturating_add(output)),
    })
}

#[tauri::command]
pub async fn get_token_usage_report(
    app: AppHandle,
    request: contracts::UsageReportRequest,
) -> Result<contracts::UsageReport, AppError> {
    let from = parse_date(&request.from_date)?;
    let to = parse_date(&request.to_date)?;
    if to < from || (to - from).num_days() > 731 {
        return Err("AI usage report range must contain at most 732 ordered days".into());
    }
    Ok((repository::report(db::pool(&app).await?, from, to).await)?)
}

fn parse_date(value: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| format!("Invalid AI usage report date: {value}"))
}
