use std::sync::Mutex;

use super::{close_writer, with_open_writer};

#[test]
fn closed_writer_rejects_late_callbacks_before_they_touch_disk() {
    let active = Mutex::new(true);
    close_writer(&active).unwrap();
    let mut wrote = false;
    let write_attempt = with_open_writer(&active, || {
        wrote = true;
        Ok(())
    });
    assert_eq!(
        write_attempt.unwrap_err(),
        "Quick Look thumbnail writer is closed"
    );
    assert!(!wrote);
}
