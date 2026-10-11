mod grant;
mod scope;

pub(crate) use grant::{create_grant, delete_unreferenced_application_grants, load_grant};
pub(crate) use scope::{
    external_agent_grant, local_user_grant, AuthorizationContext, Principal, PrincipalKind,
    RunScope,
};

pub(crate) async fn require_read(
    app: &tauri::AppHandle,
    grant_id: &str,
    space_id: &str,
) -> Result<(), String> {
    let pool = crate::spaces::db::pool(app).await?;
    let mut tx = pool.begin().await.map_err(crate::spaces::db::sql_error)?;
    let grant = load_grant(&mut tx, grant_id).await?;
    grant.require_read(space_id, crate::foundation::clock::unix_time_millis()?)
}

pub(crate) async fn require_run_control(
    app: &tauri::AppHandle,
    grant_id: &str,
    space_id: &str,
    run_id: &str,
) -> Result<(), String> {
    let pool = crate::spaces::db::pool(app).await?;
    let mut tx = pool.begin().await.map_err(crate::spaces::db::sql_error)?;
    let grant = load_grant(&mut tx, grant_id).await?;
    grant.require_read(space_id, crate::foundation::clock::unix_time_millis()?)?;
    let owner: Option<(String, Option<String>, String, String)> = sqlx::query_as(
        "SELECT scope_kind,home_space_id,principal_kind,principal_id FROM spaces_runs WHERE id=$1",
    )
    .bind(run_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(crate::spaces::db::sql_error)?;
    let Some((scope_kind, home_space_id, principal_kind, principal_id)) = owner else {
        return Err("run-unavailable: Run is unavailable".into());
    };
    let run_scope = RunScope::from_db(&scope_kind, home_space_id)?;
    if run_scope.require_space_id()? != space_id
        || grant.scope != run_scope
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
    action_ids: Vec<String>,
) -> Result<String, String> {
    let now = crate::foundation::clock::unix_time_millis()?;
    let grant = external_agent_grant(
        uuid::Uuid::new_v4().to_string(),
        space_id.to_owned(),
        actor_id.to_owned(),
        action_ids,
        now,
    );
    let pool = crate::spaces::db::pool(app).await?;
    let mut tx = crate::spaces::db::begin_write(pool).await?;
    create_grant(&mut tx, &grant, now).await?;
    tx.commit().await.map_err(crate::spaces::db::sql_error)?;
    Ok(grant.grant_id)
}
