use chrono::NaiveDate;

use super::{contracts::UsageDay, repository::measure};

fn day(date: &str, total_tokens: u64) -> UsageDay {
    UsageDay {
        date: date.to_owned(),
        input_tokens: total_tokens / 2,
        output_tokens: total_tokens - total_tokens / 2,
        total_tokens,
        requests: 1,
    }
}

#[test]
fn measures_totals_peak_and_streaks() {
    let activity = vec![
        day("2026-09-17", 100),
        day("2026-09-18", 250),
        day("2026-09-20", 300),
        day("2026-09-21", 400),
        day("2026-09-22", 500),
    ];
    let metrics = measure(
        &activity,
        NaiveDate::from_ymd_opt(2026, 9, 22).expect("valid date"),
    )
    .expect("metrics");

    assert_eq!(metrics.total_tokens, 1_550);
    assert_eq!(metrics.requests, 5);
    assert_eq!(metrics.active_days, 5);
    assert_eq!(metrics.peak_daily_tokens, 500);
    assert_eq!(metrics.current_streak_days, 3);
    assert_eq!(metrics.longest_streak_days, 3);
}

#[test]
fn current_streak_remains_active_during_the_next_day() {
    let activity = vec![day("2026-09-20", 100), day("2026-09-21", 100)];
    let metrics = measure(
        &activity,
        NaiveDate::from_ymd_opt(2026, 9, 22).expect("valid date"),
    )
    .expect("metrics");

    assert_eq!(metrics.current_streak_days, 2);
    assert_eq!(metrics.longest_streak_days, 2);
}
