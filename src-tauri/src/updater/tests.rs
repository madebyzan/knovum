use serde_json::json;
use tokio_util::sync::CancellationToken;

use super::config;
use super::contracts::{UpdateMetadata, UpdatePhase, UpdateSnapshot};
use super::state::{require_version, DownloadOutcome, UpdateState};

fn selected(phase: UpdatePhase) -> UpdateSnapshot {
    let mut snapshot = UpdateSnapshot::new(true);
    snapshot.phase = phase;
    snapshot.update = Some(UpdateMetadata {
        version: "0.0.2".into(),
        current_version: "0.0.1".into(),
        notes: Some("Reviewed release".into()),
    });
    snapshot
}

#[test]
fn empty_plugin_configuration_cannot_initialize() {
    assert!(serde_json::from_value::<tauri_plugin_updater::Config>(json!({})).is_err());
}

#[test]
fn exactly_one_endpoint_and_a_key_are_required() {
    assert!(config::validate(&tauri_plugin_updater::Config::default()).is_err());
    let value = json!({ "pubkey": "test-envelope", "endpoints": [
        "https://example.com/one.json", "https://example.com/two.json"
    ] });
    let config = serde_json::from_value(value).expect("fixture");
    assert!(config::validate(&config).is_err());
}

#[test]
fn insecure_tls_configuration_is_rejected() {
    let value = json!({ "pubkey": "test-envelope", "endpoints": ["https://example.com/latest.json"],
        "dangerousAcceptInvalidCerts": true });
    let config = serde_json::from_value(value).expect("fixture");
    assert!(config::validate(&config).is_err());
}

#[test]
fn artifact_urls_must_be_public_non_expiring_https() {
    for value in [
        "http://example.com/a",
        "https://user:password@example.com/a",
        "https://example.com/a?token=secret",
        "https://example.com/a#fragment",
    ] {
        assert!(config::validate_url(&value.parse().expect("fixture URL")).is_err());
    }
    assert!(config::validate_url(&"https://example.com/a".parse().expect("fixture URL")).is_ok());
}

#[test]
fn install_is_bound_to_the_selected_version() {
    let snapshot = selected(UpdatePhase::Ready);
    assert!(require_version(&snapshot, UpdatePhase::Ready, "0.0.2").is_ok());
    assert!(require_version(&snapshot, UpdatePhase::Ready, "0.0.3").is_err());
}

#[test]
fn install_cannot_skip_download_or_verification() {
    for phase in [
        UpdatePhase::Idle,
        UpdatePhase::Checking,
        UpdatePhase::Available,
        UpdatePhase::Downloading,
        UpdatePhase::Verifying,
        UpdatePhase::Installing,
    ] {
        assert!(require_version(&selected(phase), UpdatePhase::Ready, "0.0.2").is_err());
    }
}

#[test]
fn absent_metadata_cannot_authorize_installation() {
    let mut snapshot = UpdateSnapshot::new(true);
    snapshot.phase = UpdatePhase::Ready;
    assert!(require_version(&snapshot, UpdatePhase::Ready, "0.0.2").is_err());
}

#[tokio::test]
async fn checks_downloads_and_installs_share_one_operation_lock() {
    let state = UpdateState::new(true);
    let operation = state.operation.lock().await;
    assert!(state.operation.try_lock().is_err());
    drop(operation);
    assert!(state.operation.try_lock().is_ok());
}

#[test]
fn observations_have_monotonic_revisions_without_a_webview() {
    let state = UpdateState::new(true);
    let initial = state.snapshot().expect("snapshot");
    let next = state
        .publish(|session| session.snapshot.phase = UpdatePhase::Checking)
        .expect("publish");
    assert_eq!(next.revision, initial.revision + 1);
    assert_eq!(
        state.snapshot().expect("snapshot").phase,
        UpdatePhase::Checking
    );
}

#[test]
fn cancellation_is_accessible_while_an_operation_owns_the_gate() {
    let state = UpdateState::new(true);
    let token = CancellationToken::new();
    state
        .publish(|session| session.cancellation = Some(token.clone()))
        .expect("publish");
    let _operation = state.operation.try_lock().expect("operation");
    state
        .lock()
        .expect("session")
        .cancellation
        .as_ref()
        .expect("token")
        .cancel();
    assert!(token.is_cancelled());
}

#[test]
fn cancellation_wins_a_race_with_verified_download_completion() {
    let state = UpdateState::new(true);
    let token = CancellationToken::new();
    state
        .publish(|session| {
            session.snapshot = selected(UpdatePhase::Verifying);
            session.cancellation = Some(token.clone());
        })
        .expect("publish");
    token.cancel();
    let snapshot = state
        .complete_download(DownloadOutcome::Verified(vec![1, 2, 3]))
        .expect("complete");
    assert_eq!(snapshot.phase, UpdatePhase::Available);
    assert_eq!(snapshot.downloaded, 0);
    assert!(state.lock().expect("session").bytes.is_none());
}

#[test]
fn only_successful_verification_promotes_the_download_to_ready() {
    let state = UpdateState::new(true);
    state
        .publish(|session| session.snapshot = selected(UpdatePhase::Verifying))
        .expect("publish");
    let snapshot = state
        .complete_download(DownloadOutcome::Verified(vec![1, 2, 3]))
        .expect("complete");
    assert_eq!(snapshot.phase, UpdatePhase::Ready);
    assert_eq!(snapshot.total, Some(3));
    assert_eq!(
        state.lock().expect("session").bytes.as_ref().map(Vec::len),
        Some(3)
    );
}

#[test]
fn signature_failure_never_retains_installable_bytes() {
    let state = UpdateState::new(true);
    state
        .publish(|session| session.snapshot = selected(UpdatePhase::Verifying))
        .expect("publish");
    let snapshot = state
        .complete_download(DownloadOutcome::Failed(
            super::contracts::UpdateFailure::Signature,
        ))
        .expect("complete");
    assert_eq!(snapshot.phase, UpdatePhase::Available);
    assert!(snapshot.failure.is_some());
    assert!(state.lock().expect("session").bytes.is_none());
}
