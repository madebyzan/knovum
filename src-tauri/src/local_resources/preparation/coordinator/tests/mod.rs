use super::*;

fn installation() -> Arc<Installation> {
    Installation::new(ModelKind::FasterWhisperConfig)
}

#[tokio::test]
async fn cancelling_one_waiter_preserves_shared_installation() {
    let installation = installation();
    let mut cancelled_waiter = installation.join();
    let remaining_waiter = installation.join();
    let request_cancellation = CancellationToken::new();
    request_cancellation.cancel();

    let cancelled_preparation = cancelled_waiter
        .wait(&request_cancellation, &mut |_| {})
        .await;
    assert_eq!(
        cancelled_preparation.unwrap_err(),
        RESOURCE_PREPARATION_CANCELLED
    );
    drop(cancelled_waiter);
    assert!(!installation.cancellation.is_cancelled());

    drop(remaining_waiter);
}

#[test]
fn dropping_last_waiter_cancels_installation() {
    let installation = installation();
    let waiter = installation.join();

    drop(waiter);

    assert!(installation.cancellation.is_cancelled());
}

#[tokio::test]
async fn terminal_progress_reaches_remaining_waiter() {
    let installation = installation();
    let mut waiter = installation.join();
    installation.finish(InstallationState::Progress(ResourcePreparationProgress {
        stage: ResourcePreparationStage::Ready,
        downloaded: catalog::FASTER_WHISPER_CONFIG.download_size,
        total: Some(catalog::FASTER_WHISPER_CONFIG.download_size),
        percent: Some(100),
    }));
    let mut observed = None;

    waiter
        .wait(&CancellationToken::new(), &mut |progress| {
            observed = Some(progress)
        })
        .await
        .unwrap();

    let observed = observed.unwrap();
    assert!(matches!(observed.stage, ResourcePreparationStage::Ready));
    assert_eq!(observed.percent, Some(100));
    assert!(!installation.cancellation.is_cancelled());
}

#[test]
fn cancelled_installation_remains_registered_until_runner_finishes() {
    let kind = ModelKind::FasterWhisperConfig;
    let identity = catalog::descriptor(kind).snapshot;
    let installation = Installation::new(kind);
    let waiter = installation.join();
    installations()
        .lock()
        .unwrap()
        .insert(identity, Arc::clone(&installation));

    drop(waiter);

    let registered = installations().lock().unwrap().get(&identity).cloned();
    assert!(registered
        .as_ref()
        .is_some_and(|current| Arc::ptr_eq(current, &installation)));
    complete(
        identity,
        &installation,
        InstallationState::Failed("cancelled".into()),
    );
    assert!(!installations().lock().unwrap().contains_key(&identity));
}

#[test]
fn snapshot_parts_share_installation_but_different_snapshots_do_not() {
    let weights = catalog::descriptor(ModelKind::FasterWhisperWeights).snapshot;
    let config = catalog::descriptor(ModelKind::FasterWhisperConfig).snapshot;
    assert!(weights == config);
    assert!(catalog::descriptor(ModelKind::TextEmbeddingConfig).snapshot != config);
}

#[test]
fn combined_progress_keeps_the_full_download_budget_across_phases() {
    let progress = ResourcePreparationProgress {
        stage: ResourcePreparationStage::Downloading,
        downloaded: 100,
        total: Some(500),
        percent: Some(20),
    }
    .with_overall_progress(1000, Some(1500));
    assert_eq!(progress.downloaded, 1100);
    assert_eq!(progress.total, Some(1500));
    assert_eq!(progress.percent, Some(73));
    assert!(matches!(
        progress.stage,
        ResourcePreparationStage::Downloading
    ));
}

#[test]
fn unknown_preparation_size_remains_indeterminate() {
    let progress = ResourcePreparationProgress {
        stage: ResourcePreparationStage::RuntimeInstalling,
        downloaded: 0,
        total: None,
        percent: None,
    }
    .with_overall_progress(100, Some(1_500));
    assert_eq!(progress.downloaded, 100);
    assert_eq!(progress.total, None);
    assert_eq!(progress.percent, None);
}

#[test]
fn known_model_download_does_not_claim_an_unknown_combined_runtime_total() {
    let progress = ResourcePreparationProgress {
        stage: ResourcePreparationStage::Downloading,
        downloaded: 200,
        total: Some(500),
        percent: Some(40),
    }
    .with_overall_progress(100, None);
    assert_eq!(progress.downloaded, 300);
    assert_eq!(progress.total, None);
    assert_eq!(progress.percent, None);
}
