use tauri::{plugin::TauriPlugin, Wry};

pub fn plugin() -> TauriPlugin<Wry> {
    tauri::plugin::Builder::new("local-resources")
        .invoke_handler(tauri::generate_handler![
            crate::local_resources::get_status,
            crate::local_resources::preparation::commands::prepare_resources,
            crate::local_resources::preparation::commands::cancel_resource_preparation,
            crate::local_resources::preparation::commands::list_resource_preparations,
            crate::local_resources::models::diarization::commands::set_diarization_access_token,
            crate::local_resources::models::diarization::commands::remove_diarization_model,
        ])
        .build()
}
