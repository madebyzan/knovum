use super::{audio_error, classify_audio_error};

#[test]
fn permission_denial_uses_the_error_kind_even_with_a_localized_message() {
    let error = cpal::Error::with_message(cpal::ErrorKind::PermissionDenied, "麦克风访问被拒绝");

    assert_eq!(
        classify_audio_error(error),
        "microphone_permission_denied: 麦克风访问被拒绝"
    );
}

#[test]
fn device_unavailability_does_not_infer_permission_denial_from_message_text() {
    let error = cpal::Error::with_message(
        cpal::ErrorKind::DeviceNotAvailable,
        "No access to a disconnected device",
    );

    assert_eq!(audio_error(&error).0, "microphone_unavailable");
}
