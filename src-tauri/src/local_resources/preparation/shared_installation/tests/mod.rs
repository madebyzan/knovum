use super::*;

fn active_installation() -> (Arc<Registry<u64>>, Arc<Installation<u64>>, Waiter<u64>) {
    let registry = Arc::new(Registry {
        active: Mutex::new(None),
    });
    let installation = Installation::new(0);
    *registry.active.lock().unwrap() = Some(Arc::clone(&installation));
    let waiter = Waiter::new(Arc::clone(&registry), Arc::clone(&installation));
    (registry, installation, waiter)
}

#[test]
fn last_waiter_cancels_shared_installation() {
    let (_, installation, waiter) = active_installation();
    drop(waiter);
    assert!(installation.cancellation.is_cancelled());
}

#[test]
fn one_waiter_does_not_cancel_another_waiter() {
    let (registry, installation, first) = active_installation();
    installation.waiters.fetch_add(1, Ordering::AcqRel);
    let second = Waiter::new(registry, Arc::clone(&installation));
    drop(first);
    assert!(!installation.cancellation.is_cancelled());
    drop(second);
    assert!(installation.cancellation.is_cancelled());
}

#[tokio::test]
async fn concurrent_waiters_share_one_job_and_cancel_independently() {
    let registry = Registry::<u64>::new();
    let starts = Arc::new(AtomicUsize::new(0));
    let (finish, finished) = tokio::sync::oneshot::channel();
    let first_cancel = CancellationToken::new();
    let second_cancel = CancellationToken::new();
    let counter = Arc::clone(&starts);
    let mut first = registry
        .join(&first_cancel, 0, move |installation| async move {
            counter.fetch_add(1, Ordering::AcqRel);
            installation.publish(42);
            finished.await.map_err(|error| error.to_string())
        })
        .await
        .unwrap();
    let mut second = registry
        .join(&second_cancel, 0, |_| async {
            Err("duplicate installation".into())
        })
        .await
        .unwrap();
    first_cancel.cancel();
    assert_eq!(
        first.wait(&first_cancel, &mut |_| {}).await.unwrap_err(),
        cancelled()
    );
    let installation = Arc::clone(&first.installation);
    drop(first);
    assert!(!installation.cancellation.is_cancelled());
    finish.send(()).unwrap();
    second.wait(&second_cancel, &mut |_| {}).await.unwrap();
    assert_eq!(starts.load(Ordering::Acquire), 1);
}

#[tokio::test]
async fn failed_installation_can_be_retried() {
    let registry = Registry::<u64>::new();
    let cancellation = CancellationToken::new();
    let mut first = registry
        .join(&cancellation, 0, |_| async {
            Err("download interrupted".into())
        })
        .await
        .unwrap();
    assert_eq!(
        first.wait(&cancellation, &mut |_| {}).await.unwrap_err(),
        "download interrupted"
    );
    let mut retry = registry
        .join(&cancellation, 0, |_| async { Ok(()) })
        .await
        .unwrap();
    retry.wait(&cancellation, &mut |_| {}).await.unwrap();
}
