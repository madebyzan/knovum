mod approval;
mod grant;
mod scope;

pub(crate) use approval::{consume as consume_approval, create as create_approval};
pub(crate) use approval::{ApprovalDecision, ApprovalRequest, ApprovalSummary};

#[tauri::command]
pub async fn list_pending_action_approvals(
    app: tauri::AppHandle,
    space_id: String,
) -> Result<Vec<approval::ApprovalSummary>, String> {
    approval::list_pending(crate::spaces::db::pool(&app).await?, &space_id).await
}

#[tauri::command]
pub async fn decide_action_approval(
    app: tauri::AppHandle,
    approval_id: String,
    decision: ApprovalDecision,
) -> Result<(), String> {
    approval::decide(crate::spaces::db::pool(&app).await?, &approval_id, decision).await?;
    crate::spaces::runs::announce_change();
    Ok(())
}
pub(crate) use grant::{create_grant, load_grant};
pub(crate) use scope::{AuthorizationContext, Principal, PrincipalKind};

pub(crate) async fn require_read(
    app: &tauri::AppHandle,
    grant_id: &str,
    space_id: &str,
) -> Result<(), String> {
    let pool = crate::spaces::db::pool(app).await?;
    let mut tx = crate::spaces::db::begin_write(pool).await?;
    let grant = load_grant(&mut tx, grant_id).await?;
    grant.require_read(space_id, crate::clock::unix_time_millis()?)
}

pub(crate) async fn require_run_control(
    app: &tauri::AppHandle,
    grant_id: &str,
    space_id: &str,
    run_id: &str,
) -> Result<(), String> {
    let pool = crate::spaces::db::pool(app).await?;
    let mut tx = crate::spaces::db::begin_write(pool).await?;
    let grant = load_grant(&mut tx, grant_id).await?;
    grant.require_read(space_id, crate::clock::unix_time_millis()?)?;
    let owner: Option<(String, String, String)> = sqlx::query_as(
        "SELECT home_space_id,principal_kind,principal_id FROM spaces_runs WHERE id=$1",
    )
    .bind(run_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(crate::spaces::db::sql_error)?;
    let Some((home_space_id, principal_kind, principal_id)) = owner else {
        return Err("run-unavailable: Run is unavailable".into());
    };
    if home_space_id != space_id
        || principal_kind != grant.principal.kind.as_str()
        || principal_id != grant.principal.id
    {
        return Err("permission-denied: Run control belongs to another principal".into());
    }
    Ok(())
}

pub(crate) async fn create_external_agent_grant(
    app: &tauri::AppHandle,
    space_id: &str,
    actor_id: &str,
) -> Result<String, String> {
    let now = crate::clock::unix_time_millis()?;
    let grant = AuthorizationContext {
        principal: Principal {
            kind: PrincipalKind::ExternalAgent,
            id: actor_id.to_owned(),
        },
        grant_id: uuid::Uuid::new_v4().to_string(),
        home_space_id: space_id.to_owned(),
        read_space_ids: vec![space_id.to_owned()],
        write_space_ids: vec![space_id.to_owned()],
        action_ids: crate::spaces::actions::definitions()
            .iter()
            .map(|definition| definition.id.to_owned())
            .collect(),
        valid_until: now + 8 * 60 * 60 * 1_000,
    };
    let pool = crate::spaces::db::pool(app).await?;
    let mut tx = crate::spaces::db::begin_write(pool).await?;
    create_grant(&mut tx, &grant, now).await?;
    tx.commit().await.map_err(crate::spaces::db::sql_error)?;
    Ok(grant.grant_id)
}
