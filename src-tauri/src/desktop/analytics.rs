use std::time::Duration;

use serde::Serialize;
use url::Url;
use uuid::Uuid;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AnalyticsEvent {
    installation_id: String,
    event: &'static str,
    app_version: &'static str,
    platform: &'static str,
}

#[tauri::command]
pub async fn send_analytics_event(installation_id: String) -> Result<String, String> {
    let endpoint = match option_env!("KNOVUM_ANALYTICS_ENDPOINT") {
        Some(endpoint) if !endpoint.trim().is_empty() => endpoint,
        _ => return Ok("not_configured".into()),
    };

    let installation_id = Uuid::parse_str(&installation_id)
        .map_err(|_| "Invalid analytics installation identifier".to_string())?;
    let endpoint = Url::parse(endpoint).map_err(|_| "Invalid analytics endpoint".to_string())?;
    if endpoint.scheme() != "https"
        || endpoint.host_str().is_none()
        || !endpoint.username().is_empty()
        || endpoint.password().is_some()
    {
        return Err("Analytics endpoint must be a credential-free HTTPS URL".into());
    }

    let event = AnalyticsEvent {
        installation_id: installation_id.to_string(),
        event: "app_started",
        app_version: env!("CARGO_PKG_VERSION"),
        platform: std::env::consts::OS,
    };
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .map_err(|_| "Analytics client could not be initialized".to_string())?;
    let response = match client.post(endpoint).json(&event).send().await {
        Ok(response) => response,
        Err(_) => return Ok("unavailable".into()),
    };

    Ok(if response.status().is_success() {
        "sent"
    } else {
        "unavailable"
    }
    .into())
}
