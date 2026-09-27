use super::*;

fn active_installation() -> (Arc<Registry>, Arc<Installation>, Waiter) {
    let registry = Arc::new(Registry {
        active: Mutex::new(None),
    });
    let installation = Installation::new();
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
