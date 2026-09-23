mod occurrence;

pub(super) use occurrence::test as test_trigger;

use sqlx::SqlitePool;

use super::{
    contract::{AutomationSource, MisfirePolicy},
    repository, schedule, AutomationTrigger,
};
use crate::spaces::db;

const MAX_DUE_SCAN: usize = 1_001;

pub(crate) async fn run_due(pool: &SqlitePool) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    super::events::reconcile_action_events(pool).await?;
    super::events::dispatch_pending(pool, now).await?;
    let ids = sqlx::query_scalar::<_, String>(
        "SELECT id FROM spaces_automation_triggers WHERE enabled=1 \
         AND next_scheduled_at <= $1 ORDER BY next_scheduled_at LIMIT 20",
    )
    .bind(now)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    for id in ids {
        if let Err(error) = reserve_due(pool, &id, now).await {
            log::error!("Automation trigger {id} failed: {error}");
        }
    }
    occurrence::drain(pool, now).await
}

async fn reserve_due(pool: &SqlitePool, id: &str, now: i64) -> Result<(), String> {
    let trigger = repository::load(pool, id).await?;
    let due = due_times(&trigger, now)?;
    if due.is_empty() {
        return Ok(());
    }
    let eligible = eligible_indexes(&trigger, &due, now);
    for (index, scheduled_at) in due.iter().copied().enumerate() {
        let state = if trigger.valid_until < now {
            "authorization_expired"
        } else if eligible.contains(&index) {
            "queued"
        } else {
            "skipped_misfire"
        };
        occurrence::reserve(pool, &trigger.id, scheduled_at, state, now).await?;
    }
    let last = *due.last().expect("due was checked as non-empty");
    let AutomationSource::Cron {
        schedule: expression,
        timezone,
        ..
    } = &trigger.source
    else {
        return Err("automation-source-invalid: due scan selected a non-cron trigger".into());
    };
    let next = schedule::next_after(expression, timezone, last)?;
    repository::advance_schedule(pool, &trigger.id, last, next, now).await
}

fn due_times(trigger: &AutomationTrigger, now: i64) -> Result<Vec<i64>, String> {
    let Some(mut cursor) = trigger.next_scheduled_at else {
        return Ok(Vec::new());
    };
    let mut due = Vec::new();
    while cursor <= now && due.len() < MAX_DUE_SCAN {
        due.push(cursor);
        let AutomationSource::Cron {
            schedule: expression,
            timezone,
            ..
        } = &trigger.source
        else {
            return Err("automation-source-invalid: due scan selected a non-cron trigger".into());
        };
        cursor = schedule::next_after(expression, timezone, cursor)?;
    }
    if due.len() == MAX_DUE_SCAN && cursor <= now {
        return Err("automation-backlog-exceeded: more than 1000 occurrences are pending".into());
    }
    Ok(due)
}

fn eligible_indexes(trigger: &AutomationTrigger, due: &[i64], now: i64) -> Vec<usize> {
    let AutomationSource::Cron {
        misfire_policy,
        misfire_window_seconds,
        catch_up_limit,
        ..
    } = &trigger.source
    else {
        return Vec::new();
    };
    let cutoff = now.saturating_sub(i64::from(*misfire_window_seconds) * 1_000);
    let within = due
        .iter()
        .enumerate()
        .filter_map(|(index, scheduled)| (*scheduled >= cutoff).then_some(index))
        .collect::<Vec<_>>();
    match misfire_policy {
        MisfirePolicy::Skip | MisfirePolicy::Latest => within.last().copied().into_iter().collect(),
        MisfirePolicy::CatchUp => {
            let limit = usize::from(*catch_up_limit);
            within
                .into_iter()
                .rev()
                .take(limit)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect()
        }
    }
}
