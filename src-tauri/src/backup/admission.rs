use tauri::{AppHandle, Runtime};

use crate::{
    job_registry::JobAdmissionReservation,
    local_resources::preparation::requests::PreparationAdmissionReservation,
};

pub(super) struct MaintenanceAdmission {
    _spaces: [JobAdmissionReservation; 4],
    _extensions: JobAdmissionReservation,
    _preparation: PreparationAdmissionReservation,
}

pub(super) async fn reserve<R: Runtime>(
    app: &AppHandle<R>,
) -> Result<MaintenanceAdmission, String> {
    let admission = MaintenanceAdmission {
        _spaces: crate::spaces::backup::reserve_maintenance()?,
        _extensions: crate::extensions::reserve_plugin_operations_for_maintenance()?,
        _preparation: crate::local_resources::preparation::requests::reserve_maintenance()?,
    };
    crate::spaces::backup::prepare_package_staging(app).await?;
    let pool = crate::sqlite::connect_feature_database(app, "spaces.db").await?;
    let mut connection = pool.acquire().await.map_err(|error| error.to_string())?;
    crate::spaces::backup::assert_quiescent(&mut connection).await?;
    Ok(admission)
}
