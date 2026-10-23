use super::{physical_memory, snapshot};

#[test]
fn reads_positive_system_memory_snapshot() {
    let total = physical_memory().unwrap();
    let current = snapshot().unwrap();
    assert!(total > 0);
    assert_eq!(current.total_bytes, total);
    assert!(current.available_bytes > 0);
    assert!(current.available_bytes <= current.total_bytes);
}
