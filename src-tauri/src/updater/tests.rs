use super::{UpdateFailure, UpdateMetadata, UpdateService, UpdateSnapshot, UpdateState};

fn release() -> UpdateMetadata {
    UpdateMetadata { version: "1.2.3".into(), current_version: "1.2.2".into(), notes: "Reviewed notes".into() }
}

#[test]
fn disabled_builds_never_offer_update_operations() {
    let service = UpdateService::new(false);
    let state = service.snapshot().unwrap().state;
    assert!(!state.can_check());
    assert!(state.require_version("1.2.3", false).is_err());
    assert!(state.require_version("1.2.3", true).is_err());
}

#[test]
fn the_selected_version_is_required_for_both_download_and_install() {
    let available = UpdateState::Available { release: release() };
    let ready = UpdateState::Ready { release: release() };
    assert!(available.require_version("1.2.3", false).is_ok());
    assert!(available.require_version("1.2.4", false).is_err());
    assert!(available.require_version("1.2.3", true).is_err());
    assert!(ready.require_version("1.2.3", true).is_ok());
    assert!(ready.require_version("1.2.4", true).is_err());
    assert!(ready.require_version("1.2.3", false).is_err());
}

#[test]
fn checks_cannot_discard_pending_or_verified_updates() {
    for state in [
        UpdateState::Checking,
        UpdateState::Available { release: release() },
        UpdateState::Downloading { release: release(), downloaded: 0, total: None },
        UpdateState::Verifying { release: release() },
        UpdateState::Ready { release: release() },
        UpdateState::Installing { release: release() },
    ] { assert!(!state.can_check()); }
    for state in [UpdateState::Idle, UpdateState::Current, UpdateState::Failed { reason: UpdateFailure::Check }] {
        assert!(state.can_check());
    }
}

#[test]
fn update_operations_are_single_flight_and_release_the_lock_on_failure() {
    let service = UpdateService::new(true);
    let operation = service.operation.try_lock().unwrap();
    assert!(service.operation.try_lock().is_err());
    drop(operation);
    assert!(service.operation.try_lock().is_ok());
}

#[test]
fn progress_contract_preserves_unknown_sizes_and_camel_case_metadata() {
    let snapshot = UpdateSnapshot {
        revision: 7,
        state: UpdateState::Downloading { release: release(), downloaded: 42, total: None },
    };
    let value = serde_json::to_value(snapshot).unwrap();
    assert_eq!(value["revision"], 7);
    assert_eq!(value["state"]["status"], "downloading");
    assert_eq!(value["state"]["release"]["currentVersion"], "1.2.2");
    assert!(value["state"]["total"].is_null());
}


#[test]
fn only_replacing_application_files_blocks_an_ordinary_exit() {
    let service = UpdateService::new(true);
    assert!(!service.blocks_exit());
    service.lock().unwrap().snapshot.state = UpdateState::Ready { release: release() };
    assert!(!service.blocks_exit());
    service.lock().unwrap().snapshot.state = UpdateState::Installing { release: release() };
    assert!(service.blocks_exit());
}
