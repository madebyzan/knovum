use sqlx::SqlitePool;

mod occurrences;
mod policy;
mod row;

pub(super) use occurrences::{advance_schedule, list_occurrences};

use super::{
    contract::{AutomationSource, AutomationTrigger, SaveAutomationTriggerRequest},
    schedule,
};
use crate::spaces::{authorization::create_grant, db, workflows};
use policy::{enum_text, source_kind, trigger_grant, validate};

pub(super) async fn save(
    pool: &SqlitePool,
    request: SaveAutomationTriggerRequest,
) -> Result<AutomationTrigger, String> {
    validate(&request)?;
    let now = crate::clock::unix_time_millis()?;
    let next = match &request.source {
        AutomationSource::Cron {
            schedule: expression,
            timezone,
            ..
        } => Some(schedule::next_after(expression, timezone, now)?),
        AutomationSource::Event { .. } => None,
    };
    let workflow =
        workflows::load_workflow_version(pool, &request.workflow_id, request.workflow_version)
            .await?;
    if workflow.space_id != request.space_id {
        return Err("permission-denied: Workflow belongs to another Space".into());
    }
    let grant = trigger_grant(&request, &workflow.definition.nodes, now);
    let mut tx = db::begin_write(pool).await?;
    create_grant(&mut tx, &grant, now).await?;
    sqlx::query("INSERT INTO spaces_automation_triggers(id,space_id,name,workflow_id,workflow_version,
        grant_id,source_kind,source_config,input,overlap_policy,
        enabled,valid_until,next_scheduled_at,created_at,updated_at)
        VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$14)
        ON CONFLICT(id) DO UPDATE SET name=excluded.name,workflow_id=excluded.workflow_id,
        workflow_version=excluded.workflow_version,grant_id=excluded.grant_id,
        source_kind=excluded.source_kind,source_config=excluded.source_config,
        input=excluded.input,overlap_policy=excluded.overlap_policy,enabled=excluded.enabled,
        valid_until=excluded.valid_until,next_scheduled_at=excluded.next_scheduled_at,updated_at=excluded.updated_at")
        .bind(&request.id).bind(&request.space_id).bind(&request.name).bind(&request.workflow_id)
        .bind(request.workflow_version).bind(&grant.grant_id).bind(source_kind(&request.source))
        .bind(serde_json::to_string(&request.source).map_err(db::json_error)?)
        .bind(serde_json::to_string(&request.input).map_err(db::json_error)?)
        .bind(enum_text(request.overlap_policy)?).bind(request.enabled)
        .bind(request.valid_until).bind(next).bind(now)
        .execute(&mut *tx).await.map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    load(pool, &request.id).await
}

pub(super) async fn list(
    pool: &SqlitePool,
    space_id: &str,
) -> Result<Vec<AutomationTrigger>, String> {
    let rows = sqlx::query(
        "SELECT * FROM spaces_automation_triggers WHERE space_id=$1 ORDER BY updated_at DESC",
    )
    .bind(space_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    rows.iter().map(row::trigger).collect()
}

pub(super) async fn load(pool: &SqlitePool, id: &str) -> Result<AutomationTrigger, String> {
    let row = sqlx::query("SELECT * FROM spaces_automation_triggers WHERE id=$1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(db::sql_error)?
        .ok_or("automation-trigger-unavailable: trigger is unavailable")?;
    row::trigger(&row)
}

pub(super) async fn set_enabled(pool: &SqlitePool, id: &str, enabled: bool) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    if enabled {
        let valid_until: Option<i64> =
            sqlx::query_scalar("SELECT valid_until FROM spaces_automation_triggers WHERE id=$1")
                .bind(id)
                .fetch_optional(pool)
                .await
                .map_err(db::sql_error)?;
        if valid_until.is_none_or(|expiry| expiry < now) {
            return Err(
                "automation-authorization-expired: renew authorization before enabling".into(),
            );
        }
    }
    let result =
        sqlx::query("UPDATE spaces_automation_triggers SET enabled=$1,updated_at=$2 WHERE id=$3")
            .bind(enabled)
            .bind(now)
            .bind(id)
            .execute(pool)
            .await
            .map_err(db::sql_error)?;
    db::require_one(
        result,
        "automation-trigger-unavailable: trigger is unavailable",
    )
}

pub(super) async fn renew(pool: &SqlitePool, id: &str, valid_until: i64) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    if valid_until <= now || valid_until > now + 366 * 24 * 60 * 60 * 1_000 {
        return Err("automation-renewal-invalid: authorization must expire within one year".into());
    }
    let mut tx = db::begin_write(pool).await?;
    let row = sqlx::query("SELECT grant_id FROM spaces_automation_triggers WHERE id=$1")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(db::sql_error)?
        .ok_or("automation-trigger-unavailable: trigger is unavailable")?;
    let grant_id: String = sqlx::Row::try_get(&row, "grant_id").map_err(db::sql_error)?;
    sqlx::query(
        "UPDATE spaces_authorization_grants SET valid_until=$1 WHERE id=$2 AND revoked_at IS NULL",
    )
    .bind(valid_until)
    .bind(grant_id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    sqlx::query("UPDATE spaces_automation_triggers SET valid_until=$1,updated_at=$2 WHERE id=$3")
        .bind(valid_until)
        .bind(now)
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)
}

pub(super) async fn delete(pool: &SqlitePool, id: &str) -> Result<(), String> {
    let result = sqlx::query("DELETE FROM spaces_automation_triggers WHERE id=$1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(db::sql_error)?;
    db::require_one(
        result,
        "automation-trigger-unavailable: trigger is unavailable",
    )
}
