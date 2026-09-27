use chrono::{Days, NaiveDate};
use sqlx::{Row, SqlitePool};

use super::contracts::{UsageDay, UsageMetrics, UsagePurpose, UsagePurposeTotal, UsageReport};

pub async fn insert(
    pool: &SqlitePool,
    purpose: UsagePurpose,
    provider: &str,
    model: &str,
    usage: &crate::ai::contracts::TokenUsage,
) -> Result<(), String> {
    let now = chrono::Local::now();
    sqlx::query("INSERT INTO ai_token_usage_events(id,occurred_at,local_date,purpose,provider,model,input_tokens,output_tokens,total_tokens) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9)")
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(now.timestamp_millis())
        .bind(now.date_naive().to_string())
        .bind(purpose.as_str())
        .bind(provider)
        .bind(model)
        .bind(i64::from(usage.prompt_tokens))
        .bind(i64::from(usage.completion_tokens))
        .bind(i64::from(usage.total_tokens))
        .execute(pool).await.map_err(|error| error.to_string())?;
    Ok(())
}

pub async fn report(
    pool: &SqlitePool,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<UsageReport, String> {
    let rows = sqlx::query("SELECT local_date,SUM(input_tokens) input_tokens,SUM(output_tokens) output_tokens,SUM(total_tokens) total_tokens,COUNT(*) requests FROM ai_token_usage_events WHERE local_date BETWEEN $1 AND $2 GROUP BY local_date ORDER BY local_date")
        .bind(from.to_string()).bind(to.to_string()).fetch_all(pool).await.map_err(|error| error.to_string())?;
    let activity = rows.iter().map(parse_day).collect::<Result<Vec<_>, _>>()?;
    let purpose_rows = sqlx::query("SELECT purpose,SUM(total_tokens) total_tokens,COUNT(*) requests FROM ai_token_usage_events WHERE local_date BETWEEN $1 AND $2 GROUP BY purpose ORDER BY total_tokens DESC")
        .bind(from.to_string()).bind(to.to_string()).fetch_all(pool).await.map_err(|error| error.to_string())?;
    let by_purpose = purpose_rows
        .iter()
        .map(parse_purpose)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(UsageReport {
        from_date: from.to_string(),
        to_date: to.to_string(),
        summary: summarize(&activity, to)?,
        activity,
        by_purpose,
    })
}

fn parse_day(row: &sqlx::sqlite::SqliteRow) -> Result<UsageDay, String> {
    Ok(UsageDay {
        date: row.try_get("local_date").map_err(string_error)?,
        input_tokens: unsigned(row, "input_tokens")?,
        output_tokens: unsigned(row, "output_tokens")?,
        total_tokens: unsigned(row, "total_tokens")?,
        requests: unsigned(row, "requests")?,
    })
}

fn parse_purpose(row: &sqlx::sqlite::SqliteRow) -> Result<UsagePurposeTotal, String> {
    let value: String = row.try_get("purpose").map_err(string_error)?;
    let purpose = match value.as_str() {
        "assistant" => UsagePurpose::Assistant,
        "editor" => UsagePurpose::Editor,
        "knowledge" => UsagePurpose::Knowledge,
        "knowledge_indexing" => UsagePurpose::KnowledgeIndexing,
        "translation" => UsagePurpose::Translation,
        "visual_analysis" => UsagePurpose::VisualAnalysis,
        _ => return Err(format!("Unknown AI usage purpose: {value}")),
    };
    Ok(UsagePurposeTotal {
        purpose,
        total_tokens: unsigned(row, "total_tokens")?,
        requests: unsigned(row, "requests")?,
    })
}

pub(super) fn summarize(days: &[UsageDay], to: NaiveDate) -> Result<UsageMetrics, String> {
    let mut summary = UsageMetrics {
        input_tokens: days.iter().map(|day| day.input_tokens).sum(),
        output_tokens: days.iter().map(|day| day.output_tokens).sum(),
        total_tokens: days.iter().map(|day| day.total_tokens).sum(),
        requests: days.iter().map(|day| day.requests).sum(),
        active_days: days.len() as u64,
        peak_daily_tokens: days.iter().map(|day| day.total_tokens).max().unwrap_or(0),
        current_streak_days: 0,
        longest_streak_days: 0,
    };
    let dates = days
        .iter()
        .map(|day| NaiveDate::parse_from_str(&day.date, "%Y-%m-%d").map_err(string_error))
        .collect::<Result<std::collections::BTreeSet<_>, _>>()?;
    let current_end = if dates.contains(&to) {
        to
    } else {
        to.checked_sub_days(Days::new(1)).unwrap_or(to)
    };
    summary.current_streak_days = streak_ending_at(&dates, current_end);
    summary.longest_streak_days = longest_streak(&dates);
    Ok(summary)
}

fn streak_ending_at(dates: &std::collections::BTreeSet<NaiveDate>, mut day: NaiveDate) -> u64 {
    let mut streak = 0;
    while dates.contains(&day) {
        streak += 1;
        let Some(previous) = day.checked_sub_days(Days::new(1)) else {
            break;
        };
        day = previous;
    }
    streak
}

fn longest_streak(dates: &std::collections::BTreeSet<NaiveDate>) -> u64 {
    dates
        .iter()
        .fold((0, 0, None), |(longest, current, previous), day| {
            let consecutive = previous
                .and_then(|value: NaiveDate| value.checked_add_days(Days::new(1)))
                == Some(*day);
            let next = if consecutive { current + 1 } else { 1 };
            (longest.max(next), next, Some(*day))
        })
        .0
}

fn unsigned(row: &sqlx::sqlite::SqliteRow, field: &str) -> Result<u64, String> {
    let value: i64 = row.try_get(field).map_err(string_error)?;
    u64::try_from(value).map_err(string_error)
}

fn string_error(error: impl ToString) -> String {
    error.to_string()
}
