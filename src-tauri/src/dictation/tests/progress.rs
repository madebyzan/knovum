use super::*;
#[test]
fn preparation_keeps_the_recording_owner_and_releases_progress_before_asr() {
    let current = DictationSnapshot {
        session_id: Some("session".into()),
        target_application: Some("Editor".into()),
        ..Default::default()
    };
    let progress = ResourcePreparationProgress {
        stage: ResourcePreparationStage::Downloading,
        downloaded: 50,
        total: Some(100),
        percent: Some(50),
    };
    let preparing = update(current, progress.clone());
    assert!(matches!(
        preparing.phase,
        DictationPhase::PreparingResources
    ));
    assert_eq!(preparing.resources.as_ref().unwrap().downloaded, 50);
    assert_eq!(preparing.session_id.as_deref(), Some("session"));
    let completed = update(
        preparing,
        ResourcePreparationProgress {
            stage: ResourcePreparationStage::Ready,
            ..progress
        },
    );
    assert!(matches!(completed.phase, DictationPhase::Transcribing));
    assert!(completed.resources.is_none());
    assert_eq!(completed.target_application.as_deref(), Some("Editor"));
}
