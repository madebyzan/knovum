use super::JobRegistry;

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
fn release_rejects_a_missing_job() {
    let registry = registry();

    assert!(registry.release("missing").is_err());
}
