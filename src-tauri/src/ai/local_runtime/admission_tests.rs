use super::*;
use crate::ai::local_runtime::device_profiles::MemorySnapshot;

const GIB: u64 = 1024 * 1024 * 1024;

#[test]
fn reservation_accepts_exact_available_and_application_budget_boundaries() {
    let snapshot = MemorySnapshot {
        total_bytes: 16 * GIB,
        available_bytes: 5 * GIB,
    };
    assert_eq!(
        validate_memory_reservation(4 * GIB, "test", snapshot, 0).unwrap(),
        4 * GIB
    );
    assert_eq!(
        validate_memory_reservation(2 * GIB, "test", snapshot, 8 * GIB).unwrap(),
        10 * GIB
    );
}

#[test]
fn reservation_rejects_missing_headroom_and_exceeded_application_budget() {
    let low_available = MemorySnapshot {
        total_bytes: 16 * GIB,
        available_bytes: 5 * GIB - 1,
    };
    assert!(
        validate_memory_reservation(4 * GIB, "test", low_available, 0)
            .unwrap_err()
            .starts_with("resource_exhausted:")
    );

    let over_budget = MemorySnapshot {
        total_bytes: 16 * GIB,
        available_bytes: 8 * GIB,
    };
    assert!(
        validate_memory_reservation(4 * GIB, "test", over_budget, 8 * GIB)
            .unwrap_err()
            .contains("application budget")
    );
}

#[test]
fn reservation_rejects_arithmetic_overflow() {
    let snapshot = MemorySnapshot {
        total_bytes: u64::MAX,
        available_bytes: u64::MAX,
    };
    assert!(validate_memory_reservation(u64::MAX, "test", snapshot, 0)
        .unwrap_err()
        .contains("requirement overflowed"));
}
