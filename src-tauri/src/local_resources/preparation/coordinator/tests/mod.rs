use super::*;

fn installation() -> Arc<Installation> {
    Installation::new(ModelKind::WhisperPart(catalog::WhisperPart::SmallVad))
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
        downloaded: catalog::descriptor(ModelKind::WhisperPart(catalog::WhisperPart::SmallVad))
            .download_size,
        total: Some(
            catalog::descriptor(ModelKind::WhisperPart(catalog::WhisperPart::SmallVad))
                .download_size,
        ),
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
    let kind = ModelKind::WhisperPart(catalog::WhisperPart::SmallVad);
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
    let weights =
        catalog::descriptor(ModelKind::WhisperPart(catalog::WhisperPart::SmallWeights)).snapshot;
    let config =
        catalog::descriptor(ModelKind::WhisperPart(catalog::WhisperPart::SmallVad)).snapshot;
    assert!(weights == config);
    assert!(catalog::descriptor(ModelKind::TextEmbeddingConfig).snapshot != config);
}
