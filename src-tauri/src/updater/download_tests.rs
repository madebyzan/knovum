use tokio_util::sync::CancellationToken;

use super::{UpdateMetadata, UpdateService, UpdateState};

fn release() -> UpdateMetadata {
    UpdateMetadata {
        version: "1.2.3".into(),
        current_version: "1.2.2".into(),
        notes: "Reviewed update".into(),
    }
}

fn downloading() -> UpdateService {
    let service = UpdateService::new(true);
    {
        let mut session = service.lock().unwrap();
        session.cancellation = Some(CancellationToken::new());
        session.snapshot.state = UpdateState::Verifying { release: release() };
    }
    service
}

#[test]
fn cancellation_winning_the_lock_prevents_ready_and_discards_verified_bytes() {
    let service = downloading();
    service.lock().unwrap().cancel_download().unwrap();
    let snapshot = service
        .lock()
        .unwrap()
        .complete_download(release(), Some(vec![1, 2, 3]))
        .unwrap();
    assert!(matches!(snapshot.state, UpdateState::Available { .. }));
    assert_eq!(snapshot.revision, 1);
    let session = service.lock().unwrap();
    assert!(session.bytes.is_none());
    assert!(session.cancellation.is_none());
    assert!(session
        .snapshot
        .state
        .require_version("1.2.3", true)
        .is_err());
}

#[test]
fn completion_winning_the_lock_rejects_late_cancellation_without_changing_ready() {
    let service = downloading();
    let snapshot = service
        .lock()
        .unwrap()
        .complete_download(release(), Some(vec![1, 2, 3]))
        .unwrap();
    assert!(matches!(snapshot.state, UpdateState::Ready { .. }));
    let session = service.lock().unwrap();
    assert!(session.cancel_download().is_err());
    assert_eq!(session.bytes.as_deref(), Some([1, 2, 3].as_slice()));
    assert_eq!(session.snapshot.revision, snapshot.revision);
}

#[test]
fn a_cancelled_transfer_returns_to_the_same_selected_release() {
    let service = downloading();
    let snapshot = service
        .lock()
        .unwrap()
        .complete_download(release(), None)
        .unwrap();
    assert!(snapshot.state.require_version("1.2.3", false).is_ok());
    assert!(snapshot.state.require_version("1.2.4", false).is_err());
    assert!(service.lock().unwrap().bytes.is_none());
}

#[test]
fn cancellation_and_completion_require_an_active_download() {
    let service = UpdateService::new(true);
    let mut session = service.lock().unwrap();
    assert!(session.cancel_download().is_err());
    assert!(session.complete_download(release(), Some(vec![1])).is_err());
    assert_eq!(session.snapshot.revision, 0);
}
