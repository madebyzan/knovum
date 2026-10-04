use sqlx::SqlitePool;

pub(crate) async fn recover(pool: &SqlitePool) -> Result<bool, String> {
    super::controls::recover_expired_leases(pool).await
}

pub(crate) async fn apply_requested_controls(pool: &SqlitePool) -> Result<bool, String> {
    super::controls::apply(pool).await
}

pub(crate) async fn resume_waiting_steps(pool: &SqlitePool) -> Result<(), String> {
    super::waiting::resume(pool).await
}
