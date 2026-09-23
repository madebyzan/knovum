use std::str::FromStr;

use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::Tz;

pub(super) fn next_after(schedule: &str, timezone: &str, after: i64) -> Result<i64, String> {
    let schedule = cron::Schedule::from_str(schedule)
        .map_err(|error| format!("automation-schedule-invalid: {error}"))?;
    let timezone = Tz::from_str(timezone)
        .map_err(|_| "automation-timezone-invalid: IANA timezone is required")?;
    let after = Utc
        .timestamp_millis_opt(after)
        .single()
        .ok_or("automation-time-invalid: timestamp is outside supported range")?
        .with_timezone(&timezone);
    schedule
        .after(&after)
        .next()
        .map(|value| value.timestamp_millis())
        .ok_or_else(|| "automation-schedule-exhausted: no future occurrence".to_owned())
}

pub(super) fn preview(schedule: &str, timezone: &str, count: u8) -> Result<Vec<i64>, String> {
    if !(1..=20).contains(&count) {
        return Err("automation-preview-invalid: count must be between 1 and 20".into());
    }
    let schedule = cron::Schedule::from_str(schedule)
        .map_err(|error| format!("automation-schedule-invalid: {error}"))?;
    let timezone = Tz::from_str(timezone)
        .map_err(|_| "automation-timezone-invalid: IANA timezone is required")?;
    Ok(schedule
        .upcoming(timezone)
        .take(usize::from(count))
        .map(|value: DateTime<Tz>| value.timestamp_millis())
        .collect())
}
