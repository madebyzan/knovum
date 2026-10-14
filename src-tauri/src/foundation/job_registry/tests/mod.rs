use std::sync::{mpsc, Arc, Barrier};

use super::{JobAdmissionReservation, JobRegistry};

fn registry() -> &'static JobRegistry<&'static str> {
    Box::leak(Box::new(JobRegistry::new("Test")))
}

#[test]
fn lease_removes_its_job_when_dropped() {
    let registry = registry();
    let lease = registry
        .acquire("job-1", "running")
        .expect("job registered");

    assert_eq!(
        registry.get("job-1").expect("job remains active"),
        "running"
    );
    drop(lease);
    assert!(registry.get("job-1").is_err());
}

#[test]
fn acquire_rejects_duplicate_identifiers() {
    let registry = registry();
    let _lease = registry
        .acquire("job-1", "first")
        .expect("first job registered");

    assert!(registry.acquire("job-1", "second").is_err());
}

#[test]
fn cancellation_reports_active_ownership_without_cancelling_another_job() {
    let registry = Box::leak(Box::new(JobRegistry::new("Cancellation")));
    let (first, first_lease) = registry.acquire_cancellable("first").unwrap();
    let (second, _second_lease) = registry.acquire_cancellable("second").unwrap();
    assert!(!registry.cancel("missing").unwrap());
    assert!(registry.cancel("first").unwrap());
    assert!(first.is_cancelled());
    assert!(!second.is_cancelled());
    drop(first_lease);
    assert!(!registry.cancel("first").unwrap());
}

#[test]
fn release_rejects_a_missing_job() {
    let registry = registry();

    assert!(registry.release("missing").is_err());
}

#[test]
fn maintenance_reservation_rejects_active_jobs() {
    let registry = registry();
    let _lease = registry
        .acquire("job-1", "running")
        .expect("job registered");

    assert!(registry.reserve_maintenance().is_err());
}

#[test]
fn maintenance_reservation_closes_and_reopens_admission() {
    let registry = registry();
    let reservation = registry
        .reserve_maintenance()
        .expect("maintenance reserved");

    assert!(registry.insert("inserted", "job").is_err());
    assert!(registry.acquire("acquired", "job").is_err());
    assert!(registry.reserve_maintenance().is_err());

    drop(reservation);
    let _lease = registry
        .acquire("after-maintenance", "running")
        .expect("admission reopened");
}

#[test]
fn concurrent_reservation_and_acquire_admit_exactly_one() {
    let registry = registry();
    let start = Arc::new(Barrier::new(3));
    let finish = Arc::new(Barrier::new(3));
    let (tx, received) = mpsc::channel();

    let reservation_thread = {
        let start = start.clone();
        let finish = finish.clone();
        let tx = tx.clone();
        std::thread::spawn(move || {
            start.wait();
            let reservation = registry.reserve_maintenance();
            tx.send(reservation.is_ok())
                .expect("report reservation outcome");
            finish.wait();
            drop(reservation);
        })
    };
    let acquire_thread = {
        let start = start.clone();
        let finish = finish.clone();
        std::thread::spawn(move || {
            start.wait();
            let lease = registry.acquire("concurrent", "running");
            tx.send(lease.is_ok()).expect("report acquire outcome");
            finish.wait();
            drop(lease);
        })
    };

    start.wait();
    let first = received.recv().expect("first outcome");
    let second = received.recv().expect("second outcome");
    finish.wait();
    reservation_thread.join().expect("reservation thread");
    acquire_thread.join().expect("acquire thread");
    assert_ne!(first, second);
}

#[test]
fn partial_multi_registry_reservation_releases_prior_guards() {
    let first = registry();
    let second = registry();
    let _busy = second.acquire("busy", "running").expect("active job");

    let reservations: Result<[JobAdmissionReservation; 2], String> = (|| {
        let first = first.reserve_maintenance()?;
        let second = second.reserve_maintenance()?;
        Ok([first, second])
    })();
    assert!(reservations.is_err());
    let _lease = first
        .acquire("after-rollback", "running")
        .expect("prior reservation released");
}
