use super::{physical_memory, snapshot};

// Both entry points answer one question: how much memory can this machine actually give a worker.
// Windows also exposes the installed SMBIOS size, which is larger than that on any machine with
// firmware-reserved RAM; reading it here would make admission promise memory the OS never has.
#[test]
fn reads_positive_system_memory_snapshot() {
    let total = physical_memory().unwrap();
    let current = snapshot().unwrap();
    assert!(total > 0);
    assert_eq!(current.total_bytes, total);
    assert!(current.available_bytes > 0);
    assert!(current.available_bytes <= current.total_bytes);
}
