pub(crate) fn unix_time_millis() -> Result<i64, String> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("system clock is before Unix epoch: {error}"))?;
    i64::try_from(duration.as_millis())
        .map_err(|_| "system time exceeds supported range".to_string())
}
